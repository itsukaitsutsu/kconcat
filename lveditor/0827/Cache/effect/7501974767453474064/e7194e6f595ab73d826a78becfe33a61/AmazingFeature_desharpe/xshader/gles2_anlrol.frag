

 precision highp float;
 varying vec2 texCoord;
 varying vec2 sucaiTexCoord;
 uniform float opacity;

 uniform sampler2D u_FBOTexture;
 uniform sampler2D sucaiImageTexture;

 uniform float intensity;
 
#ifndef BLEND_MULTIPLY
#define BLEND_MULTIPLY

vec3 blendMultiply(vec3 base, vec3 blend) {
    return base * blend;
}

vec3 blendMultiply(vec3 base, vec3 blend, float opacity) {
    return (blendMultiply(base, blend) * opacity + blend * (1.0 - opacity));
}

#endif

#define blendModel blendMultiply


 void main(void)
 {
    //  vec4 src = texture2D(u_FBOTexture, texCoord);
    //  vec4 sucai = texture2D(sucaiImageTexture, sucaiTexCoord);

    //  vec3 color = blendModel(src.rgb, clamp(sucai.rgb * (1.0 / sucai.a), 0.0, 1.0));
    //  //float alpha = sucai.a * intensity;
    //  //color *= alpha;
    //  //color = mix(src.rgb, color, sucai.a);
    //  //color = mix(src.rgb, color, intensity);
    //  //gl_FragColor = vec4(color, alpha);

    //  color = mix(src.rgb, color, sucai.a);
    //  color = mix(src.rgb, color, intensity * opacity);
    //  gl_FragColor = vec4(color, 1.0);

    float mask = texture2D(sucaiImageTexture, sucaiTexCoord).r;
    gl_FragColor = vec4(mask, mask, mask, 1.0);
 }
 