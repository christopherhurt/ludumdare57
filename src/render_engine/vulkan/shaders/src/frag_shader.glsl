#version 450

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

void main() {
    // TODO: hardcoded directional light
    vec3 lightDir = normalize(vec3(-0.25, -0.5, -1.0));
    float diffuseDot = max(dot(fragNormal, -lightDir), 0.0);
    float ambientFactor = 0.2;

    //outColor = vec4(fragColor.rgb * (diffuseDot + ambientFactor), fragColor.a);
    outColor = fragColor; // TODO: use fragTexCoord
}
