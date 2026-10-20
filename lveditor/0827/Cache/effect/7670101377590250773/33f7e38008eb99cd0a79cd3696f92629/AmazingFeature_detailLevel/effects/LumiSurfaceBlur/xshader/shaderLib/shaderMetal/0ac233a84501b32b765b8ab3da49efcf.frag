#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float u_threshold;
    float u_stepY;
    float u_stepX;
};

struct main0_out
{
    float4 o_fragColor [[color(0)]];
};

struct main0_in
{
    float2 uv [[user(locn0)]];
};

static inline __attribute__((always_inline))
float4 _f0(thread float4& _p0, thread const float4& _p1, thread const float4& _p2)
{
    float _23;
    if (_p1.x < 9.9999997473787516355514526367188e-06)
    {
        _23 = _p2.x;
    }
    else
    {
        _23 = _p0.x / _p1.x;
    }
    _p0.x = _23;
    float _40;
    if (_p1.y < 9.9999997473787516355514526367188e-06)
    {
        _40 = _p2.y;
    }
    else
    {
        _40 = _p0.y / _p1.y;
    }
    _p0.y = _40;
    float _57;
    if (_p1.z < 9.9999997473787516355514526367188e-06)
    {
        _57 = _p2.z;
    }
    else
    {
        _57 = _p0.z / _p1.z;
    }
    _p0.z = _57;
    float _74;
    if (_p1.w < 9.9999997473787516355514526367188e-06)
    {
        _74 = _p2.w;
    }
    else
    {
        _74 = _p0.w / _p1.w;
    }
    _p0.w = _74;
    return _p0;
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_inputTexture [[texture(0)]], sampler u_inputTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    float4 _t0 = float4(0.0);
    float4 _t1 = float4(0.0);
    float4 _104 = u_inputTexture.sample(u_inputTextureSmplr, in.uv);
    float _109 = fast::max(buffer.u_threshold, 9.9999997473787516355514526367188e-06);
    for (int _t4 = -7; _t4 <= 7; _t4++)
    {
        float _127 = float(_t4) * buffer.u_stepY;
        for (int _t6 = -7; _t6 <= 7; _t6++)
        {
            float4 _152 = u_inputTexture.sample(u_inputTextureSmplr, (in.uv + float2(float(_t6) * buffer.u_stepX, _127)));
            float4 _167 = fast::max(float4(1.0) - (abs(_152 - _104) / float4(2.5 * _109)), float4(0.0));
            _t0 += (_152 * _167);
            _t1 += _167;
        }
    }
    float4 param = _t0;
    float4 param_1 = _t1;
    float4 param_2 = _104;
    float4 _187 = _f0(param, param_1, param_2);
    _t0 = _187;
    out.o_fragColor = fast::clamp(_187, float4(0.0), float4(1.0));
    return out;
}

