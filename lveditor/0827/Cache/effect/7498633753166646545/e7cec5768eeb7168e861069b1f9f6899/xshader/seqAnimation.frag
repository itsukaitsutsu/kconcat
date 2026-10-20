#version 300 es
precision highp float;

layout(location = 0) out vec4 FragColor;

in vec2 o_uv;

uniform sampler2D underline_1;
uniform vec3 bgColor;
uniform int colorIns;
#define ProjectionScale 3.


void main(void) {

    vec4 seqColor = texture(underline_1, o_uv);
    FragColor = seqColor;
    //   if(colorIns==0){
    //     FragColor = seqColor;
    // } else {
    //     FragColor.rgb=bgColor*seqColor.a;
    // }
    // FragColor.a = 0.5;
}