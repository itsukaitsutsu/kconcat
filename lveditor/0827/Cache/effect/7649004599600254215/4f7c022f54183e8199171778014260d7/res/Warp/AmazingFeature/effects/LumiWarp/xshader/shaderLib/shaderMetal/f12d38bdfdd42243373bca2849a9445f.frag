#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float4 u_ScreenParams;
    int u_axisFrag;
    int u_warpStyleFrag;
    float u_bend;
};

struct main0_out
{
    float4 o_fragColor [[color(0)]];
};

struct main0_in
{
    float2 v_uv [[user(locn0)]];
};

static inline __attribute__((always_inline))
float2 _f3(thread const float2& _p0, thread float& _p1, thread const int& _p2)
{
    if (_p2 != 0)
    {
        _p1 *= (-1.0);
    }
    float2 _t1 = _p0;
    float _80 = (1.0 / pow(fast::clamp(abs(_p1), 9.9999997473787516355514526367188e-06, 1.0), 0.569999992847442626953125)) - 1.0;
    _t1.x -= 0.5;
    if (_p1 < 0.0)
    {
        _t1.y = 1.0 - _t1.y;
    }
    _t1.y += _80;
    float _107 = _t1.y;
    float _109 = _t1.x;
    float _114 = precise::atan2(0.5, _80);
    _t1.y = (length(_t1) - (sqrt(0.25 + (_80 * _80)) - _80)) - _80;
    if (_p1 < 0.0)
    {
        _t1.y = 1.0 - _t1.y;
    }
    _t1.x = 1.0 - ((precise::atan2(_107, _109) - (1.57079637050628662109375 - _114)) / (_114 * 2.0));
    if (_p2 == 0)
    {
        return _t1;
    }
    else
    {
        return float2(_t1.y, _t1.x);
    }
}

static inline __attribute__((always_inline))
float2 _f0(thread const float2& _p0)
{
    return float2(1.0) - _p0;
}

static inline __attribute__((always_inline))
float _f1(thread const float& _p0)
{
    return _p0 * _p0;
}

static inline __attribute__((always_inline))
float _f2(thread const float& _p0)
{
    return (_p0 * _p0) * _p0;
}

static inline __attribute__((always_inline))
float2 _f4(thread const float2& _p0, thread const float& _p1, thread const int& _p2, thread const bool& _p3, constant float4& u_ScreenParams)
{
    float _172 = u_ScreenParams.x / u_ScreenParams.y;
    float2 _t7 = _p0;
    if (_p3)
    {
        float2 param = _t7;
        _t7 = _f0(param);
    }
    float _184 = _t7.x - 0.5;
    float _191 = (1.0 / pow(fast::clamp(abs(_p1), 9.9999997473787516355514526367188e-06, 1.0), 0.569999992847442626953125)) - 1.0;
    float param_1 = _191;
    float _198 = sqrt(_f1(param_1) + 0.25);
    float param_2 = _198;
    float param_3 = _184;
    float _t11 = ((sqrt(_f1(param_2) - _f1(param_3)) - _191) * _172) + 1.0;
    if (_p1 < 0.0)
    {
        float param_4 = _198;
        float param_5 = _184;
        _t11 = 1.0 - ((sqrt(_f1(param_4) - _f1(param_5)) - _191) * _172);
    }
    _t7.y /= _t11;
    _t7.x -= 0.5;
    float param_6 = abs((abs(_t7.x) * 2.0) - 0.5);
    _t7.x *= mix(1.0, (_f2(param_6) * 2.0) + 0.75, _t7.y * fast::clamp(abs(_p1), 0.0, 1.0));
    _t7.x += 0.5;
    if (_p3)
    {
        float2 param_7 = _t7;
        _t7 = _f0(param_7);
    }
    if (_p2 != 0)
    {
        return float2(_t7.y, _t7.x);
    }
    return _t7;
}

static inline __attribute__((always_inline))
float _f5(thread const float2& _p0)
{
    float2 _t13 = step(float2(0.0), _p0) * step(_p0, float2(1.0));
    return _t13.x * _t13.y;
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_inputTex [[texture(0)]], sampler u_inputTexSmplr [[sampler(0)]])
{
    main0_out out = {};
    float2 _t14 = in.v_uv;
    bool _308 = buffer.u_axisFrag == 0;
    float _t15;
    float _t16;
    if (_308)
    {
        _t15 = _t14.x;
        _t16 = _t14.y;
    }
    else
    {
        _t15 = _t14.y;
        _t16 = _t14.x;
    }
    float2 _325 = float2(_t15, _t16);
    if (buffer.u_warpStyleFrag == 0)
    {
        float2 param = _325;
        float param_1 = buffer.u_bend;
        int param_2 = buffer.u_axisFrag;
        float2 _338 = _f3(param, param_1, param_2);
        _t14 = _338;
    }
    else
    {
        if (buffer.u_warpStyleFrag == 1)
        {
            float2 param_3 = _325;
            float param_4 = buffer.u_bend;
            int param_5 = buffer.u_axisFrag;
            bool param_6 = _308;
            _t14 = _f4(param_3, param_4, param_5, param_6, buffer.u_ScreenParams);
        }
        else
        {
            if (buffer.u_warpStyleFrag == 2)
            {
                float2 param_7 = _325;
                float param_8 = buffer.u_bend;
                int param_9 = buffer.u_axisFrag;
                bool param_10 = buffer.u_axisFrag != 0;
                _t14 = _f4(param_7, param_8, param_9, param_10, buffer.u_ScreenParams);
            }
            else
            {
                if (buffer.u_warpStyleFrag == 5)
                {
                    _t14.x = 1.0 - _t14.x;
                    if (buffer.u_bend > 0.0)
                    {
                        if (_308)
                        {
                            _t14.y = 1.0 - _t14.y;
                        }
                        else
                        {
                            if (buffer.u_axisFrag == 1)
                            {
                                _t14.x = 1.0 - _t14.x;
                            }
                        }
                    }
                }
                else
                {
                    if (buffer.u_warpStyleFrag == 6)
                    {
                        if (_308 && (buffer.u_bend <= 0.0))
                        {
                            _t14.y = 1.0 - _t14.y;
                        }
                        if (buffer.u_axisFrag == 1)
                        {
                            _t14.y = 1.0 - _t14.y;
                            if (buffer.u_bend > 0.0)
                            {
                                _t14.x = 1.0 - _t14.x;
                            }
                        }
                    }
                }
            }
        }
    }
    float2 param_11 = _t14;
    out.o_fragColor = u_inputTex.sample(u_inputTexSmplr, _t14) * _f5(param_11);
    return out;
}

