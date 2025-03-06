use anyhow::{anyhow, Result};
use log::info;
use std::collections::HashSet;
use vulkanalia::{Device as vk_Device, Version};
use vulkanalia::bytecode::Bytecode;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{self, ExtDebugUtilsExtension, KhrSwapchainExtension};
use vulkanalia::window as vk_window;
use winit::window::Window as winit_Window;

use crate::render_engine::vulkan::vulkan_structs::{BufferResources, ImageResources, Pipeline, Swapchain, Vertex};
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

const VULKAN_PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
const VALIDATION_LAYER_NAME: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const REQUIRED_DEVICE_EXTENSION_NAMES: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

// Instance

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
        .api_version(vk::make_version(1, 0, 0));

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    let flags = if cfg!(target_os = "macos") && entry.version()? >= VULKAN_PORTABILITY_MACOS_VERSION {
        info!("Enabling extensions for macOS portability.");

        extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
        extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());

        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::empty()
    };

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

unsafe fn create_shader_module(shader_path: &str, device: &Device) -> Result<vk::ShaderModule> {
    let bytes = include_bytes!(shader_path); // TODO: runtime load
    let bytecode = Bytecode::new(bytes).unwrap();

    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());

    Ok(device.create_shader_module(&info, None)?)
}

unsafe fn create_render_pass(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    format: vk::Format,
) -> Result<vk::RenderPass> {
    let color_attachment = vk::AttachmentDescription::builder()
        .format(format)
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

unsafe fn create_pipeline(
    device: &Device,
    render_pass: vk::RenderPass,
    swapchain: &Swapchain,
    descriptor_set_layout: vk::DescriptorSetLayout,
) -> Result<Pipeline> {
    let vert_shader_module = create_shader_module("../shaders/generated/vert.spv", device)?; // TODO: update path
    let frag_shader_module = create_shader_module("../shaders/generated/frag.spv", device)?; // TODO: update path

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

unsafe fn create_color_objects(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    swapchain: Swapchain,
) -> Result<(ImageResources, vk::ImageView)> {
    let color_image_resources = create_image(
        instance,
        device,
        physical_device,
        swapchain.extent.width,
        swapchain.extent.height,
        swapchain.format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    let color_image_view = create_image_view(
        device,
        color_image_resources.image,
        swapchain.format,
        vk::ImageAspectFlags::COLOR,
    )?;

    Ok((color_image_resources, color_image_view))
}

unsafe fn create_depth_objects(
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

unsafe fn create_framebuffer(
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

unsafe fn create_vertex_buffer(
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

unsafe fn create_index_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    indices: &Vec<u32>,
) -> Result<BufferResources> {
    let size = (size_of::<u32>() * indices.len()) as u64;

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

    destroy_buffer(device, staging_buffer_resources);

    Ok(index_buffer_resources)
}

unsafe fn create_uniform_buffer<T: Sized>(
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
