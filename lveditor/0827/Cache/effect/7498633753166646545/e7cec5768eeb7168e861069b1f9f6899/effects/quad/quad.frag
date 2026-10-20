#version 300 es
precision highp float;

layout(location = 0) out vec4 FragColor;

in vec2 o_uv;

uniform sampler2D _MainTex;
uniform sampler2D underline_1;
uniform vec4 AABB;
uniform float u_Alpha;
uniform float u_Scale;
uniform vec3 bgColor;
#define ProjectionScale 3.
uniform vec2 u_textRect;
uniform vec2 textRect;
uniform float fontSize;


void main(void) {

    vec4 text = texture(_MainTex, o_uv);
    // seqColor.rgb = clamp(seqColor.rgb/seqColor.a,0.0,1.0);
    // FragColor = vec4(blendFunc(text.rgb,seqColor.rgb,seqColor.a,1),1.0);
    FragColor = text;
    // FragColor.a =text;
}