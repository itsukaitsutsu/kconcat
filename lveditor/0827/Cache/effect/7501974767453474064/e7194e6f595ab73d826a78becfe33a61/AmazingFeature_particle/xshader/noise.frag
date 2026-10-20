precision highp float;
varying vec2 uv0;
uniform sampler2D inputImageTexture;

uniform float Intensity;
uniform float Seed;
const float Scale = 9331.99;


float getNoise(vec2 uv, float seed, float intensity)
{
    vec3 xyz = vec3(uv * Scale, seed * Scale);
    xyz  = fract(xyz * .1031) + seed;
    xyz += dot(xyz, xyz.yzx + 33.33);
    float noise = fract((xyz.x + xyz.y) * xyz.z);
    noise = (noise - 0.5) * intensity;
    return noise;
}


void main(void) 
{
    vec4 color = texture2D(inputImageTexture, uv0);
    color.rgb += getNoise(uv0, Seed, Intensity);
    color.rgb = clamp(color.rgb, 0.0, 1.0);
    gl_FragColor = color;
}
