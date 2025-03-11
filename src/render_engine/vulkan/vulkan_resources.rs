use anyhow::{anyhow, Result};
use log::{info, warn};
use std::collections::HashSet;
use vulkanalia::Device as vk_Device;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{self, ExtDebugUtilsExtension, KhrSwapchainExtension};
use vulkanalia::window as vk_window;
use winit::window::Window as winit_Window;

use crate::render_engine::vulkan::vulkan_structs::{BufferResources, FrameSyncObjects, ImageResources, Pipeline, Swapchain, Vertex};
use crate::render_engine::vulkan::vulkan_utils::{
    copy_buffer,
    debug_callback,
    destroy_buffer,
    get_depth_format,
    get_memory_type_index,
    get_queue_family_indices,
    get_swapchain_extent,
    get_swapchain_present_mode,
    get_swapchain_support,
    get_swapchain_surface_format,
    transition_image_layout,
};

const VALIDATION_LAYER_NAME: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const REQUIRED_DEVICE_EXTENSION_NAMES: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

// Instance + Devices

pub(in crate::render_engine::vulkan) unsafe fn create_vk_instance(
    window: &winit_Window,
    entry: &Entry,
    debug_enabled: bool,
) -> Result<(Instance, Option<vk::DebugUtilsMessengerEXT>)> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"My Cool Game\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"Hurt Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 3, 216));

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    let flags = vk::InstanceCreateFlags::empty();

    let available_layers = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();
    let mut enabled_layers: Vec<*const i8> = Vec::new();

    if debug_enabled {
        if !available_layers.contains(&VALIDATION_LAYER_NAME) {
            return Err(anyhow!("Validation layers are enabled but not supported."));
        }

        enabled_layers.push(VALIDATION_LAYER_NAME.as_ptr());
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&enabled_layers)
        .enabled_extension_names(&extensions)
        .flags(flags);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .user_callback(Some(debug_callback));

    if debug_enabled {
        info = info.push_next(&mut debug_info);
    }

    let instance = entry.create_instance(&info, None)?;

    let debug_messenger = if debug_enabled {
        Some(instance.create_debug_utils_messenger_ext(&debug_info, None)?)
    } else {
        None
    };

    Ok((instance, debug_messenger))
}

pub(in crate::render_engine::vulkan) unsafe fn pick_physical_device(instance: &Instance, surface: vk::SurfaceKHR) -> Result<vk::PhysicalDevice> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        let is_suitable = is_suitable_physical_device(instance, surface, physical_device);
        if let Err(error) = is_suitable {
            warn!("Skipping physical device (`{}`): {}", properties.device_name, error);
        } else if !is_suitable.unwrap() {
            warn!("Skipping unsuitable physical device (`{}`)", properties.device_name);
        } else {
            info!("Selected physical device (`{}`).", properties.device_name);
            return Ok(physical_device);
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
}

unsafe fn is_suitable_physical_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<bool> {
    // TODO: prefer properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU

    if get_queue_family_indices(instance, surface, physical_device).is_err() {
        return Ok(false);
    }

    if !has_required_physical_device_extensions(instance, physical_device)? {
        return Ok(false);
    }

    let support = get_swapchain_support(instance, surface, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Ok(false);
    }

    Ok(true)
}

unsafe fn has_required_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<bool> {
    let supported_extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();

    Ok(REQUIRED_DEVICE_EXTENSION_NAMES.iter().all(|e| supported_extensions.contains(e)))
}

pub(in crate::render_engine::vulkan) unsafe fn create_logical_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    debug_enabled: bool,
) -> Result<(Device, vk::Queue, vk::Queue)> {
    let indices = get_queue_family_indices(instance, surface, physical_device)?;

    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    let layers = if debug_enabled {
        vec![VALIDATION_LAYER_NAME.as_ptr()]
    } else {
        vec![]
    };

    let extensions = REQUIRED_DEVICE_EXTENSION_NAMES
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();

    let features = vk::PhysicalDeviceFeatures::builder();

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = instance.create_device(physical_device, &info, None)?;

    let graphics_queue = device.get_device_queue(indices.graphics, 0);
    let present_queue = device.get_device_queue(indices.present, 0);

    Ok((device, graphics_queue, present_queue))
}

// Image + Image Views

unsafe fn create_image(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<ImageResources> {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D { width, height, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::_1)
        .flags(vk::ImageCreateFlags::empty());

    let image = device.create_image(&info, None)?;

    let requirements = device.get_image_memory_requirements(image);

    let info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(
            get_memory_type_index(
                instance,
                physical_device,
                properties,
                requirements,
            )?
        );

    let memory = device.allocate_memory(&info, None)?;

    device.bind_image_memory(image, memory, 0)?;

    Ok(ImageResources { image, memory })
}

unsafe fn create_image_view(
    device: &vk_Device,
    image: vk::Image,
    format: vk::Format,
    aspects: vk::ImageAspectFlags,
) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);

    Ok(device.create_image_view(&info, None)?)
}

// Swapchain

pub(in crate::render_engine::vulkan) unsafe fn create_swapchain(
    preferred_image_count: u32,
    window: &winit_Window,
    instance: &Instance,
    surface: vk::SurfaceKHR,
    device: &vk_Device,
    physical_device: vk::PhysicalDevice,
) -> Result<Swapchain> {
    let indices = get_queue_family_indices(instance, surface, physical_device)?;
    let support = get_swapchain_support(instance, surface, physical_device)?;

    let surface_format = get_swapchain_surface_format(&support.formats);
    let present_mode = get_swapchain_present_mode(&support.present_modes);
    let extent = get_swapchain_extent(window, support.capabilities);

    let image_count = if preferred_image_count < support.capabilities.min_image_count {
        support.capabilities.min_image_count
    } else if preferred_image_count > support.capabilities.max_image_count && support.capabilities.max_image_count != 0 {
        support.capabilities.max_image_count
    } else {
        preferred_image_count
    };

    let (queue_family_indices, image_sharing_mode) = if indices.graphics != indices.present {
        (vec![indices.graphics, indices.present], vk::SharingMode::CONCURRENT)
    } else {
        (vec![], vk::SharingMode::EXCLUSIVE)
    };

    let info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    let swapchain = device.create_swapchain_khr(&info, None)?;
    let swapchain_images = device.get_swapchain_images_khr(swapchain)?;
    let swapchain_image_views = swapchain_images
        .iter()
        .map(|i| create_image_view(device, *i, surface_format.format, vk::ImageAspectFlags::COLOR))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(
        Swapchain {
            format: surface_format.format,
            extent,
            swapchain,
            images: swapchain_images,
            image_views: swapchain_image_views,
        }
    )
}

// Render Pass + Pipeline

unsafe fn create_shader_module(device: &Device, bytes: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytes).unwrap();

    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());

    Ok(device.create_shader_module(&info, None)?)
}

pub(in crate::render_engine::vulkan) unsafe fn create_render_pass(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    swapchain_format: vk::Format,
) -> Result<vk::RenderPass> {
    let color_attachment = vk::AttachmentDescription::builder()
        .format(swapchain_format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let depth_stencil_attachment = vk::AttachmentDescription::builder()
        .format(get_depth_format(instance, physical_device)?)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];
    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments)
        .depth_stencil_attachment(&depth_stencil_attachment_ref);

    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

    let attachments = &[color_attachment, depth_stencil_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];
    let info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);

    Ok(device.create_render_pass(&info, None)?)
}

pub(in crate::render_engine::vulkan) unsafe fn create_pipeline(
    device: &Device,
    render_pass: vk::RenderPass,
    swapchain: &Swapchain,
    descriptor_set_layout: vk::DescriptorSetLayout,
) -> Result<Pipeline> {
    let vert_shader_bytes = include_bytes!("shaders/generated/vert_shader.spv");
    let frag_shader_bytes = include_bytes!("shaders/generated/frag_shader.spv");

    let vert_shader_module = create_shader_module(device, vert_shader_bytes)?;
    let frag_shader_module = create_shader_module(device, frag_shader_bytes)?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain.extent.width as f32)
        .height(swapchain.extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);
    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(swapchain.extent);

    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::_1);

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let set_layouts=  &[descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts);

    let pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .min_depth_bounds(0.0)
        .max_depth_bounds(1.0)
        .stencil_test_enable(false);

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .depth_stencil_state(&depth_stencil_state)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);

    let pipeline = device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?.0[0];

    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);

    Ok(
        Pipeline {
            pipeline,
            layout: pipeline_layout,
        }
    )
}

// Framebuffers + Attachments

pub(in crate::render_engine::vulkan) unsafe fn create_depth_objects(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    swapchain_extent: &vk::Extent2D,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
) -> Result<(ImageResources, vk::ImageView)> {
    let format = get_depth_format(instance, physical_device)?;

    let depth_image_resources = create_image(
        instance,
        device,
        physical_device,
        swapchain_extent.width,
        swapchain_extent.height,
        format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    let depth_image_view = create_image_view(
        device,
        depth_image_resources.image,
        format,
        vk::ImageAspectFlags::DEPTH,
    )?;

    transition_image_layout(
        device,
        command_pool,
        graphics_queue,
        depth_image_resources.image,
        format,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    )?;

    Ok((depth_image_resources, depth_image_view))
}

pub(in crate::render_engine::vulkan) unsafe fn create_framebuffer(
    device: &Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    swapchain_image_view: vk::ImageView,
    depth_image_view: vk::ImageView,
) -> Result<vk::Framebuffer> {
    let attachments = &[swapchain_image_view, depth_image_view];
    let create_info = vk::FramebufferCreateInfo::builder()
        .render_pass(render_pass)
        .attachments(attachments)
        .width(extent.width)
        .height(extent.height)
        .layers(1);

    Ok(device.create_framebuffer(&create_info, None)?)
}

// Buffers

unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<BufferResources> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None)?;

    let requirements = device.get_buffer_memory_requirements(buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(
            get_memory_type_index(
                instance,
                physical_device,
                properties,
                requirements,
            )?
        );

    let memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(buffer, memory, 0)?;

    Ok(BufferResources { buffer, memory })
}

pub(in crate::render_engine::vulkan) unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    vertices: &Vec<Vertex>,
) -> Result<BufferResources> {
    let size = (size_of::<Vertex>() * vertices.len()) as u64;

    let staging_buffer_resources = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(
        staging_buffer_resources.memory,
        0,
        size,
        vk::MemoryMapFlags::empty(),
    )?;

    std::ptr::copy_nonoverlapping(vertices.as_ptr(), memory.cast(), vertices.len());

    device.unmap_memory(staging_buffer_resources.memory);

    let vertex_buffer_resources = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer(device, command_pool, queue, staging_buffer_resources.buffer, vertex_buffer_resources.buffer, size)?;

    destroy_buffer(device, staging_buffer_resources)?;

    Ok(vertex_buffer_resources)
}

pub(in crate::render_engine::vulkan) unsafe fn create_index_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    indices: &Vec<usize>,
) -> Result<BufferResources> {
    let size = (size_of::<usize>() * indices.len()) as u64;

    let staging_buffer_resources = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(
        staging_buffer_resources.memory,
        0,
        size,
        vk::MemoryMapFlags::empty(),
    )?;

    std::ptr::copy_nonoverlapping(indices.as_ptr(), memory.cast(), indices.len());

    device.unmap_memory(staging_buffer_resources.memory);

    let index_buffer_resources = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer(device, command_pool, queue, staging_buffer_resources.buffer, index_buffer_resources.buffer, size)?;

    destroy_buffer(device, staging_buffer_resources)?;

    Ok(index_buffer_resources)
}

pub(in crate::render_engine::vulkan) unsafe fn create_uniform_buffer<T: Sized>(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
) -> Result<BufferResources> {
    Ok(
        create_buffer(
            instance,
            device,
            physical_device,
            size_of::<T>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?
    )
}

// Command Pool + Command Buffers

pub(in crate::render_engine::vulkan) unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<vk::CommandPool> {
    let indices = get_queue_family_indices(instance, surface, physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(indices.graphics);

    Ok(device.create_command_pool(&info, None)?)
}

pub(in crate::render_engine::vulkan) unsafe fn create_command_buffer(device: &Device, command_pool: vk::CommandPool) -> Result<vk::CommandBuffer> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    Ok(device.allocate_command_buffers(&allocate_info)?[0])
}

// Sync Objects

pub(in crate::render_engine::vulkan) unsafe fn create_sync_objects(
    device: &Device,
) -> Result<FrameSyncObjects> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED);

    Ok(
        FrameSyncObjects {
            image_available_semaphore: device.create_semaphore(&semaphore_info, None)?,
            render_finished_semaphore: device.create_semaphore(&semaphore_info, None)?,
            in_flight_fence: device.create_fence(&fence_info, None)?,
        }
    )
}

// Descriptor Sets

pub(in crate::render_engine::vulkan) unsafe fn create_descriptor_set_layout(
    device: &Device,
) -> Result<vk::DescriptorSetLayout> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let bindings = &[ubo_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(bindings);

    Ok(device.create_descriptor_set_layout(&info, None)?)
}

pub(in crate::render_engine::vulkan) unsafe fn create_descriptor_pool(
    device: &Device,
    pool_size: usize,
) -> Result<vk::DescriptorPool> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(pool_size as u32);

    let pool_sizes = &[ubo_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(pool_size as u32);

    Ok(device.create_descriptor_pool(&info, None)?)
}

pub(in crate::render_engine::vulkan) unsafe fn create_descriptor_sets(
    device: &Device,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    uniform_buffers: &Vec<BufferResources>,
) -> Result<Vec<vk::DescriptorSet>> {
    let layouts = vec![descriptor_set_layout; uniform_buffers.len()];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    let descriptor_sets = device.allocate_descriptor_sets(&info)?;

    for i in 0..descriptor_sets.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffers[i].buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        device.update_descriptor_sets(
            &[ubo_write],
            &[] as &[vk::CopyDescriptorSet],
        );
    }

    Ok(descriptor_sets)
}
