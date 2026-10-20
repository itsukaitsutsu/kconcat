#version 300 es
precision highp float;

layout(location = 0) out vec4 FragColor;

in vec2 o_uv;

uniform sampler2D _MainTex;
uniform sampler2D u_picture1;
uniform sampler2D u_sequenceTex;

uniform float u_alphaIns;
uniform vec2 u_pos;
uniform vec2 u_pos2;
uniform float u_progress;
uniform float u_scale;
uniform float u_rotation;
uniform float u_feather;
const float EPS = 1e-5;

float uvProtect(vec2 uvTemp)
{
    return step(0., uvTemp.x) * step(0., uvTemp.y) * step(uvTemp.x, 1.0) * step(uvTemp.y, 1.0);
}

void main(void) 
{   
    vec4 seqColor = texture(u_sequenceTex, o_uv);
    FragColor = seqColor;
}