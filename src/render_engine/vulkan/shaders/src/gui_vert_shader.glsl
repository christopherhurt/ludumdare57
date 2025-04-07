#version 450

layout(binding = 0) uniform GuiUniformBufferObject {
    mat4 screen;
} ubo;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 _inNormal;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out vec2 fragTexCoord;

void main() {
    gl_Position = ubo.screen * vec4(inPosition.xy, 0.0, 1.0);
    fragTexCoord = inTexCoord;
}
