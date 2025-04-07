#version 450

layout(binding = 1) uniform sampler2D texSampler;

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragTexCoord;
layout(location = 3) in float fragDepth;

layout(location = 0) out vec4 outColor;

const vec3 FOG_COLOR = vec3(0.0, 0.0, 0.0);
const float FOG_START = 50.0;
const float FOG_MAX = 120.0;

void main() {
    // TODO: hardcoded directional light
    // vec3 lightDir = normalize(vec3(-0.25, -0.5, -1.0));
    // float diffuseDot = max(dot(fragNormal, -lightDir), 0.0);
    // float ambientFactor = 0.2;

    vec4 texColor = texture(texSampler, fragTexCoord);

    if (texColor.a < 0.01) {
        discard;
    }

    float fogAmt = 0.0;
    if (fragDepth >= FOG_START && fragDepth <= FOG_MAX) {
        fogAmt = (fragDepth - FOG_START) / (FOG_MAX - FOG_START);
    } else if (fragDepth > FOG_MAX) {
        fogAmt = 1.0;
    }

    vec4 finalColor = vec4(fogAmt * FOG_COLOR + (1.0 - fogAmt) * texColor.xyz, 1.0);

    //outColor = vec4(fragColor.rgb * (diffuseDot + ambientFactor), fragColor.a);
    // outColor = fragColor;
    outColor = finalColor;
}
