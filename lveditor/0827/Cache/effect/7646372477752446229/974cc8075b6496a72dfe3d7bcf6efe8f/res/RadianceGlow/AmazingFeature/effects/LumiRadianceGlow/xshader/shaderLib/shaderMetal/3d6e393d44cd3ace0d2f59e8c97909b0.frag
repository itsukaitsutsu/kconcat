#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float4 u_ScreenParams;
    float u_sample;
    float u_sigma;
    float u_spaceDither;
    float u_stepX;
    float u_stepY;
    int u_borderType;
    float u_exposure;
};

struct main0_out
{
    float4 o_fragColor [[color(0)]];
};

struct main0_in
{
    float2 uv0 [[user(locn0)]];
};

static inline __attribute__((always_inline))
float _f2(thread const float& _p0, thread const float& _p1)
{
    return exp((((-0.5) * _p0) * _p0) / (_p1 * _p1));
}

static inline __attribute__((always_inline))
float _f1(thread const float2& _p0, constant float4& u_ScreenParams)
{
    float3 _55 = fract(float3((_p0 * u_ScreenParams.xy).xyx) * 0.103100001811981201171875);
    float3 _t1 = _55 + float3(dot(_55, _55.yzx + float3(33.3300018310546875)));
    return fma(fract(fract((_t1.x + _t1.y) * _t1.z)), 2.0, -1.0);
}

static inline __attribute__((always_inline))
float _f0(thread float& _p0)
{
    _p0 = abs(_p0);
    return abs(fma(floor(ceil(_p0) * 0.5), 2.0, -_p0));
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_inputTexture [[texture(0)]], sampler u_inputTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    if (buffer.u_sample < 9.9999997473787516355514526367188e-06)
    {
        out.o_fragColor = u_inputTexture.sample(u_inputTextureSmplr, in.uv0);
        return out;
    }
    float param = 0.0;
    float param_1 = buffer.u_sigma;
    float _130 = _f2(param, param_1);
    float _t4 = _130;
    float4 _t5 = u_inputTexture.sample(u_inputTextureSmplr, in.uv0) * _130;
    float2 _t6 = in.uv0;
    if (buffer.u_spaceDither > 9.9999997473787516355514526367188e-06)
    {
        float2 param_2 = in.uv0;
        _t6 += (float2(buffer.u_stepX, buffer.u_stepY) * (buffer.u_spaceDither * _f1(param_2, buffer.u_ScreenParams)));
    }
    float2 _t8 = _t6;
    for (int _t9 = 1; _t9 <= 1024; _t9++)
    {
        float _173 = float(_t9);
        if (_173 > buffer.u_sample)
        {
            break;
        }
        float2 _185 = float2(buffer.u_stepX, buffer.u_stepY) * _173;
        float param_3 = length(_185);
        float param_4 = buffer.u_sigma;
        float _194 = _f2(param_3, param_4);
        _t8 = _t6 - _185;
        bool _200 = _t8.x < 0.0;
        bool _207;
        if (!_200)
        {
            _207 = _t8.y < 0.0;
        }
        else
        {
            _207 = _200;
        }
        bool _214;
        if (!_207)
        {
            _214 = _t8.x > 1.0;
        }
        else
        {
            _214 = _207;
        }
        bool _221;
        if (!_214)
        {
            _221 = _t8.y > 1.0;
        }
        else
        {
            _221 = _214;
        }
        if (_221)
        {
            if (buffer.u_borderType == 0)
            {
                _t4 += _194;
            }
            else
            {
                if (buffer.u_borderType == 1)
                {
                    float param_5 = _t8.x;
                    float _242 = _f0(param_5);
                    _t8.x = _242;
                    float param_6 = _t8.y;
                    float _247 = _f0(param_6);
                    _t8.y = _247;
                    _t5 += (u_inputTexture.sample(u_inputTextureSmplr, _t8) * _194);
                    _t4 += _194;
                }
            }
        }
        else
        {
            _t5 += (u_inputTexture.sample(u_inputTextureSmplr, _t8) * _194);
            _t4 += _194;
        }
        _t8 = _t6 + _185;
        bool _275 = _t8.x < 0.0;
        bool _282;
        if (!_275)
        {
            _282 = _t8.y < 0.0;
        }
        else
        {
            _282 = _275;
        }
        bool _289;
        if (!_282)
        {
            _289 = _t8.x > 1.0;
        }
        else
        {
            _289 = _282;
        }
        bool _296;
        if (!_289)
        {
            _296 = _t8.y > 1.0;
        }
        else
        {
            _296 = _289;
        }
        if (_296)
        {
            if (buffer.u_borderType == 0)
            {
                _t4 += _194;
            }
            else
            {
                if (buffer.u_borderType == 1)
                {
                    float param_7 = _t8.x;
                    float _314 = _f0(param_7);
                    _t8.x = _314;
                    float param_8 = _t8.y;
                    float _319 = _f0(param_8);
                    _t8.y = _319;
                    _t5 += (u_inputTexture.sample(u_inputTextureSmplr, _t8) * _194);
                    _t4 += _194;
                }
            }
        }
        else
        {
            _t5 += (u_inputTexture.sample(u_inputTextureSmplr, _t8) * _194);
            _t4 += _194;
        }
    }
    float4 _345 = _t5;
    float4 _347 = _345 / float4(_t4);
    _t5 = _347;
    float3 _352 = _347.xyz * buffer.u_exposure;
    _t5.x = _352.x;
    _t5.y = _352.y;
    _t5.z = _352.z;
    out.o_fragColor = fast::clamp(_t5, float4(0.0), float4(1.0));
    return out;
}

