precision highp float;

#define SAMPLES 13
#define PI 3.14159265359

uniform sampler2D _MainTex;
uniform vec2 u_size;
uniform vec2 u_threshold;
uniform float u_kernel;

varying vec2 v_uv;
varying vec2 v_xy;

const vec3 c_brightness = vec3(0.299, 0.587, 0.114);

void main()
{
    vec2 r_size = 1.0 / u_size;
    float aR = 0.0;
    float dR = PI / float(SAMPLES);
    float maxDiff = 0.0;
    vec4 selColor0 = vec4(0.0);
    vec4 selColor1 = vec4(0.0);
    for (int i = 0; i < SAMPLES; ++i) {
        vec2 dir = vec2(cos(aR), sin(aR)) * u_kernel;
        vec4 c0 = texture2D(_MainTex, (v_xy + dir) * r_size);
        vec4 c1 = texture2D(_MainTex, (v_xy - dir) * r_size);

        float diff = abs(c0.a - c1.a);
        float select = step(maxDiff, diff);
        selColor0 = mix(selColor0, c0, select);
        selColor1 = mix(selColor1, c1, select);
        maxDiff = max(diff, maxDiff);

        aR += dR;
    }

    float b0 = dot(selColor0.rgb, c_brightness);
    float b1 = dot(selColor1.rgb, c_brightness);
    vec4 color = mix(selColor0, selColor1, step(b0, b1));
    float alpha = smoothstep(u_threshold.x, u_threshold.y, maxDiff);

    gl_FragColor = color * alpha;
}
