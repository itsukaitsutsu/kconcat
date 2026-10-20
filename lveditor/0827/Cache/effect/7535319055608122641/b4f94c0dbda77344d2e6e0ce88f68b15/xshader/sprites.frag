precision highp float;

uniform sampler2D _MainTex;
uniform sampler2D _EdgeTex;
uniform sampler2D _MaskTex;
uniform sampler2D _AnimTex;
uniform float u_textVisible;
uniform float u_edgeOpacity;
uniform vec4 u_rect;

varying vec2 v_uvText;
varying vec2 v_uvMask;
varying vec2 v_uvAnim;


vec4 texture2Dclamp (sampler2D tex, vec2 uv) {
    vec4 color = texture2D(tex, uv);
    uv = step(vec2(0.0), uv) * step(uv, vec2(1.0));
    return color * uv[0] * uv[1];
}

vec4 blendNormal (vec4 src, vec4 dst) {
    return src + dst * (1.0 - src.a);
}

void main ()
{
    vec4 colorText = texture2D(_MainTex, v_uvText);
    vec4 colorEdge = texture2D(_EdgeTex, v_uvText);
    vec4 colorMask = texture2Dclamp(_MaskTex, v_uvMask);
    vec4 colorAnim = texture2Dclamp(_AnimTex, v_uvAnim);

//    vec4 colorResult = vec4(0.0);
    vec4 colorResult = colorMask * (1.0 - colorText.a);

    colorText *= 1.0 - colorMask.a;
    colorText *= u_textVisible;
    colorResult = blendNormal(colorText, colorResult);

    colorEdge *= colorMask.a * u_edgeOpacity;
    colorResult = blendNormal(colorEdge, colorResult);

    colorResult = blendNormal(colorAnim, colorResult);

//    vec2 inRect = step(u_rect.xy, v_uvText) * step(v_uvText, u_rect.zw);
//    vec4 colorRect = mix(vec4(0.0), vec4(1.0, 0.0, 0.0, 0.5), inRect.x * inRect.y);
//    colorResult = blendNormal(colorRect, colorResult);

    gl_FragColor = colorResult;
}
