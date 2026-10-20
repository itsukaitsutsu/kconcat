precision highp float;

uniform sampler2D _FinalTex;

varying vec2 v_uv;

void main()
{
    gl_FragColor = texture2D(_FinalTex, v_uv);
}
