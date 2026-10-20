precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;

void main()
{
    vec4 oriColor = texture2D(u_inputTexture, uv0);
    float darkColor = min(min(oriColor.r, oriColor.g), oriColor.b);
    oriColor.rgb = vec3(darkColor);
    gl_FragColor = oriColor;
}
