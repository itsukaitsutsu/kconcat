precision highp float;

uniform sampler2D _ResultTex;
uniform vec2 u_offsetPoint0;
uniform vec2 u_offsetPoint1;
uniform float u_offsetRadius;
uniform float u_offsetLength;

varying vec2 v_uv;

void main()
{
    vec2 vA = u_offsetPoint1 - u_offsetPoint0;
    vec2 vB = v_uv - u_offsetPoint0;
    vec2 nA = normalize(vec2(-vA.y, vA.x));
    float dis = abs(dot(nA, vB));

    vec2 uv0 = v_uv;
    vec2 uv1 = v_uv - normalize(vA) * u_offsetLength;
    vec2 uv = mix(uv0, uv1, step(dis, u_offsetRadius));

    gl_FragColor = texture2D(_ResultTex, uv);
}
