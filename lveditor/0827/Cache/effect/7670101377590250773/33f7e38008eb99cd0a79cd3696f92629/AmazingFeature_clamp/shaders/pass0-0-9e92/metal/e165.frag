#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct main0_out
{
    float4 gl_FragColor [[color(0)]];
};

struct main0_in
{
    float2 uv [[user(uv)]];
};

fragment main0_out main0(main0_in in [[stage_in]], texture2d<float> inputImageTexture [[texture(0)]], sampler inputImageTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    out.gl_FragColor = fast::clamp(inputImageTexture.sample(inputImageTextureSmplr, in.uv), float4(0.0), float4(1.0));
    return out;
}

