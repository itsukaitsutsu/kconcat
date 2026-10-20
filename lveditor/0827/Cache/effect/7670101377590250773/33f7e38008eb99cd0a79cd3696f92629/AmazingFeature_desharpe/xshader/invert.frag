precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;

void main()
{
    vec4 oriColor = texture2D(u_inputTexture, uv0);
    oriColor.rgb = vec3(vec3(1.0) - oriColor.rgb);
    float gray = oriColor.r * 0.299 + oriColor.g * 0.587 + oriColor.b * 0.114;
    gl_FragColor = vec4(gray, gray, gray, oriColor.a);
}
