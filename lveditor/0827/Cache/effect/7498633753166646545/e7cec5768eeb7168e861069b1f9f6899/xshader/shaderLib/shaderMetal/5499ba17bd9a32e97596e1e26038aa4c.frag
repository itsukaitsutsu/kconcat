#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float scaleX;
};

struct main0_out
{
    float4 o_fragColor [[color(0)]];
};

struct main0_in
{
    float2 v_uv [[user(locn0)]];
};

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_InputTexture [[texture(0)]], sampler u_InputTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    float2 _t0 = in.v_uv - float2(0.5);
    _t0.x /= buffer.scaleX;
    float2 _27 = _t0;
    float2 _29 = _27 + float2(0.5);
    _t0 = _29;
    out.o_fragColor = u_InputTexture.sample(u_InputTextureSmplr, _29);
    return out;
}

