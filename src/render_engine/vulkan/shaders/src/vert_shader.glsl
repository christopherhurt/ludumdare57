#version 450

// NOTE: Memory layout must match the UniformBufferObject struct
layout(binding = 0) uniform UniformBufferObject {
    mat4 world;
    mat4 view;
    mat4 proj;
    vec4 color;
} ubo;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec2 fragTexCoord;

void main() {
    gl_Position = ubo.proj * ubo.view * ubo.world * vec4(inPosition, 1.0);
    fragColor = ubo.color;
    fragNormal = normalize(mat3(transpose(inverse(ubo.world))) * inNormal);
    fragTexCoord = inTexCoord;
}
