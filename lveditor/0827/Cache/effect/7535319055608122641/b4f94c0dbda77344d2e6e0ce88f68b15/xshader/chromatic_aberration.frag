precision highp float;

uniform sampler2D _StrokeTex;
uniform vec2 u_size;
uniform vec2 u_offset;

varying vec2 v_uv;
varying vec2 v_xy;

void main()
{
    vec2 rSize = 1.0 / u_size;
    vec4 R = texture2D(_StrokeTex, (v_xy + u_offset) * rSize);
    vec4 G = texture2D(_StrokeTex, v_uv);
    vec4 B = texture2D(_StrokeTex, (v_xy - u_offset) * rSize);
    vec4 color = vec4(R.r, G.g, B.b, (R.a + G.a + B.a) / 3.0);
    gl_FragColor = color;
}
