#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    int u_warpStyleVert;
    float u_bend;
    int u_axisVert;
};

struct main0_out
{
    float2 v_uv [[user(locn0)]];
    float4 gl_Position [[position]];
};

struct main0_in
{
    float2 a_position [[attribute(0)]];
    float2 a_texcoord0 [[attribute(1)]];
};

static inline __attribute__((always_inline))
float _f0(thread const float& _p0)
{
    return (_p0 * 0.5) + 0.5;
}

static inline __attribute__((always_inline))
float2 _f1(thread const float2& _p0)
{
    float param = _p0.x;
    float param_1 = _p0.y;
    return float2(_f0(param), _f0(param_1));
}

static inline __attribute__((always_inline))
float _f2(thread const float& _p0)
{
    return _p0 * _p0;
}

static inline __attribute__((always_inline))
float2 _f4(thread const float2& _p0, thread float& _p1, thread const int& _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    float2 _t65 = _p0;
    float2 param = _p0;
    float2 _t66 = _f1(param);
    float _705 = _p1;
    float _709 = (1.0 / fast::clamp(abs(_705), 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
    float _t68 = 1.0;
    float param_1 = 1.0;
    float param_2 = _709;
    float _719 = sqrt(_f2(param_1) + _f2(param_2));
    if (_p2 == 0)
    {
        _t68 = (-cos(((2.0 * _t66.x) * precise::atan2(_t68, _709)) + precise::atan2(_709, _t68))) * _719;
        float param_3 = _719;
        float param_4 = _t68;
        _t65.x = _t68;
        _t65.y += ((sqrt(_f2(param_3) - _f2(param_4)) - _709) * sign(_p1));
    }
    else
    {
        _p1 = -_p1;
        _t68 = (-cos(((2.0 * _t66.y) * precise::atan2(_t68, _709)) + precise::atan2(_709, _t68))) * _719;
        float param_5 = _719;
        float param_6 = _t68;
        _t65.y = _t68;
        _t65.x += ((sqrt(_f2(param_5) - _f2(param_6)) - _709) * sign(_p1));
    }
    return _t65;
}

static inline __attribute__((always_inline))
float2 _f5(thread const float2& _p0, thread const float& _p1, thread const int& _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    float2 _t72 = _p0;
    float2 param = _p0;
    float2 _t73 = _f1(param);
    float _820 = (1.0 / fast::clamp(abs(_p1), 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
    float _t75 = 1.0;
    float param_1 = 1.0;
    float param_2 = _820;
    float _830 = sqrt(_f2(param_1) + _f2(param_2));
    if (_p2 == 0)
    {
        _t75 = (-cos(((2.0 * _t73.x) * precise::atan2(_t75, _820)) + precise::atan2(_820, _t75))) * _830;
        float param_3 = _830;
        float param_4 = _t75;
        _t72.x = _t75;
        _t72.y += (((sqrt(_f2(param_3) - _f2(param_4)) - _820) * sign(_p1)) * _t72.y);
    }
    else
    {
        _t75 = (-cos(((2.0 * _t73.y) * precise::atan2(_t75, _820)) + precise::atan2(_820, _t75))) * _830;
        float param_5 = _830;
        float param_6 = _t75;
        _t72.y = _t75;
        _t72.x += (((sqrt(_f2(param_5) - _f2(param_6)) - _820) * sign(_p1)) * _t72.x);
    }
    return _t72;
}

static inline __attribute__((always_inline))
float2 _f3(thread const float2& _p0, thread const float& _p1, thread const int& _p2)
{
    float2 _t0 = _p0;
    if (_p1 > 0.0)
    {
        if (_p2 == 0)
        {
            float2 param = _t0;
            float2 _t1 = _f1(param);
            float2 _130 = float2(_t0.x, -1.0);
            float _137 = (1.0 / fast::clamp(_p1, 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
            float _143 = (smoothstep(0.0, 1.0, _p1) * 2.0) + 1.0;
            float _t4 = _143;
            float param_1 = _143;
            float param_2 = _137;
            float _152 = sqrt(_f2(param_1) + _f2(param_2));
            _t4 = (-cos(((2.0 * _t1.x) * precise::atan2(_t4, _137)) + precise::atan2(_137, _t4))) * _152;
            float param_3 = _152;
            float param_4 = _t4;
            float _190 = ((sqrt(_f2(param_3) - _f2(param_4)) - _137) + (_p1 + 2.0)) - (_152 - _137);
            float2 _197 = float2(_t4, _190 - 1.0);
            float2 _207 = fast::normalize(_197 - _130) * ((_190 * _p1) + 2.0);
            _t0 = ((_130 + (float2(0.0, 2.0) * _t1.y)) + ((((((_130 * (-3.0)) - float2(0.0, 4.0)) - _207) + (_197 * 3.0)) * _t1.y) * _t1.y)) + (((((((_130 * 2.0) + float2(0.0, 2.0)) + _207) - (_197 * 2.0)) * _t1.y) * _t1.y) * _t1.y);
        }
        else
        {
            float2 param_5 = _t0;
            float2 _t17 = _f1(param_5);
            float2 _270 = float2(-1.0, _t0.y);
            float _275 = (1.0 / fast::clamp(_p1, 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
            float _280 = (smoothstep(0.0, 1.0, _p1) * 2.0) + 1.0;
            float _t20 = _280;
            float param_6 = _280;
            float param_7 = _275;
            float _289 = sqrt(_f2(param_6) + _f2(param_7));
            _t20 = (-cos(((2.0 * _t17.y) * precise::atan2(_t20, _275)) + precise::atan2(_275, _t20))) * _289;
            float param_8 = _289;
            float param_9 = _t20;
            float _327 = ((sqrt(_f2(param_8) - _f2(param_9)) - _275) + (_p1 + 2.0)) - (_289 - _275);
            float2 _334 = float2(_327 - 1.0, _t20);
            float2 _344 = fast::normalize(_334 - _270) * ((_327 * _p1) + 2.0);
            _t0 = -(((_270 + (float2(2.0, 0.0) * _t17.x)) + ((((((_270 * (-3.0)) - float2(4.0, 0.0)) - _344) + (_334 * 3.0)) * _t17.x) * _t17.x)) + (((((((_270 * 2.0) + float2(2.0, 0.0)) + _344) - (_334 * 2.0)) * _t17.x) * _t17.x) * _t17.x));
        }
    }
    else
    {
        if (_p2 == 0)
        {
            float2 param_10 = _t0;
            float2 _t33 = _f1(param_10);
            float2 _410 = float2(_t0.x, -1.0);
            float _416 = (1.0 / fast::clamp(abs(_p1), 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
            float _t36 = 1.0;
            float param_11 = 1.0;
            float param_12 = _416;
            float _426 = sqrt(_f2(param_11) + _f2(param_12));
            _t36 = (-cos(((2.0 * _t33.x) * precise::atan2(_t36, _416)) + precise::atan2(_416, _t36))) * _426;
            float param_13 = _426;
            float param_14 = _t36;
            float _452 = sqrt(_f2(param_13) - _f2(param_14)) - _416;
            float2 _467 = float2(_t36, 1.0 - _452);
            float2 _t43 = float2(_t36, _452) - float2(0.0, -_416);
            _t43.y = _t43.y;
            float2 _479 = _t43;
            float2 _491 = mix(fast::normalize(_479) * ((-pow(_452, 0.5)) + 2.0), float2(0.0, -2.0), float2(1.0 - abs(_p1)));
            _t43 = _491;
            _t0 = ((_467 + (_491 * _t33.y)) + ((((((_467 * (-3.0)) - (_491 * 2.0)) - float2(0.0, -2.0)) + (_410 * 3.0)) * _t33.y) * _t33.y)) + (((((((_467 * 2.0) + _491) + float2(0.0, -2.0)) - (_410 * 2.0)) * _t33.y) * _t33.y) * _t33.y);
        }
        else
        {
            float2 param_15 = _t0;
            float2 _t49 = _f1(param_15);
            float2 _552 = float2(-1.0, _t0.y);
            float _558 = (1.0 / fast::clamp(abs(_p1), 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
            float _t52 = 1.0;
            float param_16 = 1.0;
            float param_17 = _558;
            float _568 = sqrt(_f2(param_16) + _f2(param_17));
            _t52 = (-cos(((2.0 * _t49.y) * precise::atan2(_t52, _558)) + precise::atan2(_558, _t52))) * _568;
            float param_18 = _568;
            float param_19 = _t52;
            float _594 = sqrt(_f2(param_18) - _f2(param_19)) - _558;
            float2 _608 = float2(1.0 - _594, _t52);
            float2 _t59 = float2(_t52, _594) - float2(-_558, 0.0);
            _t59.x = _t59.x;
            float2 _620 = _t59;
            float2 _634 = mix(fast::normalize(_620) * ((-pow(_t52, 0.5)) + 2.0), float2(-2.0, -0.0), float2(1.0 - abs(_p1)));
            _t59 = _634;
            _t0 = -(((_608 + (_634 * _t49.x)) + ((((((_608 * (-3.0)) - (_634 * 2.0)) - float2(-2.0, 0.0)) + (_552 * 3.0)) * _t49.x) * _t49.x)) + (((((((_608 * 2.0) + _634) + float2(-2.0, 0.0)) - (_552 * 2.0)) * _t49.x) * _t49.x) * _t49.x));
        }
    }
    return _t0;
}

static inline __attribute__((always_inline))
float2 _f6(thread const float2& _p0, thread const float& _p1, thread const int& _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    float2 _t79 = _p0;
    float2 param = _p0;
    float2 _t80 = _f1(param);
    if (_p2 == 0)
    {
        _t79.y -= ((sin((_t80.x * 3.141592502593994140625) * 2.0) * _p1) * 1.14999997615814208984375);
    }
    else
    {
        _t79.x -= ((sin((_t80.y * 3.141592502593994140625) * 2.0) * _p1) * 1.14999997615814208984375);
    }
    return _t79;
}

static inline __attribute__((always_inline))
float2 _f7(thread const float2& _p0, thread const float& _p1, thread const int& _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    float2 _t83 = _p0;
    float2 param = _p0;
    float2 _t84 = _f1(param);
    if (_p2 != 0)
    {
        _t83.x -= (((sin((_t84.y * 3.141592502593994140625) * 2.0) * _p1) * 0.64999997615814208984375) * sin(_t84.x * 3.141592502593994140625));
    }
    else
    {
        _t83.y -= (((sin((_t84.x * 3.141592502593994140625) * 2.0) * _p1) * 0.64999997615814208984375) * sin(_t84.y * 3.141592502593994140625));
    }
    return _t83;
}

static inline __attribute__((always_inline))
float2 _f8(thread const float2& _p0, thread float& _p1, thread const int& _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    float2 _t87 = _p0;
    float2 param = _p0;
    float2 _t88 = _f1(param);
    if (_p2 == 0)
    {
        _t87.y += (((sin((_t88.x * 3.141592502593994140625) * 2.0) * _p1) * _t87.y) * 1.10000002384185791015625);
    }
    else
    {
        _p1 = -_p1;
        _t87.x += (((sin((_t88.y * 3.141592502593994140625) * 2.0) * _p1) * _t87.x) * 1.10000002384185791015625);
    }
    return _t87;
}

static inline __attribute__((always_inline))
float2 _f9(thread const float2& _p0, thread const float& _p1, thread const int& _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    float2 _t91 = _p0;
    float2 param = _p0;
    float2 _t92 = _f1(param);
    if (_p2 == 0)
    {
        _t91.y -= ((smoothstep(1.0, 0.0, _t92.x) * 4.0) * _p1);
    }
    else
    {
        _t91.x -= ((smoothstep(0.0, 1.0, _t92.y) * 4.0) * (-_p1));
    }
    return _t91;
}

static inline __attribute__((always_inline))
float2 _f13(thread const float2& _p0, thread const float& _p1)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    float2 _t103 = _p0;
    float2 param = _p0;
    float2 _t104 = _f1(param);
    float _1291 = sin(_t104.x * 3.141592502593994140625) * sin(_t104.y * 3.141592502593994140625);
    float _t106 = mix(1.0, 0.25, abs(_p1) * _1291);
    if (_p1 > 0.0)
    {
        _t106 = mix(1.0, 1.89999997615814208984375, abs(_p1) * _1291);
    }
    float2 _1309 = _t103;
    float2 _1311 = _1309 * _t106;
    _t103 = _1311;
    return _1311;
}

static inline __attribute__((always_inline))
float2 _f14(thread const float2& _p0, thread const float& _p1)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    float2 _t107 = _p0;
    float2 param = _p0;
    float _t109 = (1.0 / fast::clamp(abs(_p1 * _t107.y), 9.9999997473787516355514526367188e-05, 1.0)) - 0.100000001490116119384765625;
    float param_1 = 1.0;
    float param_2 = _t109;
    float param_3 = sqrt(_f2(param_1) + _f2(param_2));
    float param_4 = _t107.x;
    float _1358 = _t109;
    _t109 = (1.0 / fast::clamp(abs(_p1 * _t107.x), 9.9999997473787516355514526367188e-05, 1.0)) - 0.100000001490116119384765625;
    float param_5 = 1.0;
    float param_6 = _t109;
    float param_7 = sqrt(_f2(param_5) + _f2(param_6));
    float _1380 = _t107.y;
    float _1382 = _t107.y;
    _t107.y += (((sqrt(_f2(param_3) - _f2(param_4)) - _1358) * sign(_t107.y)) * sign(_p1));
    _t107.x += (((sqrt(_f2(param_7) - (_1380 * _1382)) - _t109) * sign(_t107.x)) * sign(_p1));
    return _t107;
}

static inline __attribute__((always_inline))
float2 _f10(thread const float2& _p0, thread const float& _p1)
{
    float _1137 = (_p1 / 180.0) * 3.141592502593994140625;
    float _1142 = cos(_1137);
    float _1144 = sin(_1137);
    return float2x2(float2(_1142, _1144), float2(-_1144, _1142)) * _p0;
}

static inline __attribute__((always_inline))
float2 _f11(thread const float2& _p0, thread float& _p1, thread const int& _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    _p1 = -_p1;
    if (_p2 != 0)
    {
        _p1 = -_p1;
    }
    float2 _t97 = _p0;
    float2 param = _p0;
    float2 _t98 = _f1(param);
    float _1188 = sin(_t98.x * 3.141592502593994140625) * sin(_t98.y * 3.141592502593994140625);
    float2 _1196 = _t97;
    float2 _1198 = _1196 * mix(1.0, 0.75, abs(_p1) * _1188);
    _t97 = _1198;
    float2 param_1 = _1198;
    float param_2 = (_p1 * _1188) * 45.0;
    float2 _1207 = _f10(param_1, param_2);
    _t97 = _1207;
    return _1207;
}

static inline __attribute__((always_inline))
float2 _f12(thread const float2& _p0, thread float& _p1, thread const float2& _p2, thread const int& _p3)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0 * _p2;
    }
    float2 _1223 = _p0 * _p2;
    float2 _t101 = _1223;
    float2 param = _1223;
    float2 _1231 = (_f1(param) - float2(0.5)) * 2.0;
    float2 _t102 = (float2(1.0) - (_1231 * _1231)) * 0.5;
    if (_p3 != 0)
    {
        _p1 = -_p1;
    }
    _t101.y += ((_t102.x * _p1) * _t101.y);
    _t101.x -= ((_t102.y * _p1) * _t101.x);
    return _t101;
}

vertex main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer)
{
    main0_out out = {};
    float2 _t113 = in.a_position;
    if (buffer.u_warpStyleVert == 0)
    {
    }
    else
    {
        if (buffer.u_warpStyleVert == 1)
        {
        }
        else
        {
            if (buffer.u_warpStyleVert == 2)
            {
            }
            else
            {
                if (buffer.u_warpStyleVert == 3)
                {
                    float2 param = _t113;
                    float param_1 = buffer.u_bend;
                    int param_2 = buffer.u_axisVert;
                    float2 _1455 = _f4(param, param_1, param_2);
                    _t113 = _1455;
                }
                else
                {
                    if (buffer.u_warpStyleVert == 4)
                    {
                        float2 param_3 = _t113;
                        float param_4 = buffer.u_bend;
                        int param_5 = buffer.u_axisVert;
                        _t113 = _f5(param_3, param_4, param_5);
                    }
                    else
                    {
                        if (buffer.u_warpStyleVert == 5)
                        {
                            float2 param_6 = _t113;
                            float param_7 = buffer.u_bend;
                            int param_8 = buffer.u_axisVert;
                            _t113 = -_f3(param_6, param_7, param_8);
                        }
                        else
                        {
                            if (buffer.u_warpStyleVert == 6)
                            {
                                float2 param_9 = _t113;
                                float param_10 = buffer.u_bend;
                                int param_11 = buffer.u_axisVert;
                                _t113 = _f3(param_9, param_10, param_11);
                            }
                            else
                            {
                                if (buffer.u_warpStyleVert == 7)
                                {
                                    float2 param_12 = _t113;
                                    float param_13 = buffer.u_bend;
                                    int param_14 = buffer.u_axisVert;
                                    _t113 = _f6(param_12, param_13, param_14);
                                }
                                else
                                {
                                    if (buffer.u_warpStyleVert == 8)
                                    {
                                        float2 param_15 = _t113;
                                        float param_16 = buffer.u_bend;
                                        int param_17 = buffer.u_axisVert;
                                        _t113 = _f7(param_15, param_16, param_17);
                                    }
                                    else
                                    {
                                        if (buffer.u_warpStyleVert == 9)
                                        {
                                            float2 param_18 = _t113;
                                            float param_19 = buffer.u_bend;
                                            int param_20 = buffer.u_axisVert;
                                            float2 _1534 = _f8(param_18, param_19, param_20);
                                            _t113 = _1534;
                                        }
                                        else
                                        {
                                            if (buffer.u_warpStyleVert == 10)
                                            {
                                                float2 param_21 = _t113;
                                                float param_22 = buffer.u_bend;
                                                int param_23 = buffer.u_axisVert;
                                                _t113 = _f9(param_21, param_22, param_23);
                                            }
                                            else
                                            {
                                                if (buffer.u_warpStyleVert == 11)
                                                {
                                                    float2 param_24 = _t113;
                                                    float param_25 = buffer.u_bend;
                                                    _t113 = _f13(param_24, param_25);
                                                }
                                                else
                                                {
                                                    if (buffer.u_warpStyleVert == 12)
                                                    {
                                                        float2 param_26 = _t113;
                                                        float param_27 = buffer.u_bend;
                                                        _t113 = _f14(param_26, param_27);
                                                    }
                                                    else
                                                    {
                                                        if (buffer.u_warpStyleVert == 13)
                                                        {
                                                            float2 param_28 = _t113;
                                                            float param_29 = buffer.u_bend;
                                                            int param_30 = buffer.u_axisVert;
                                                            float2 _1582 = _f11(param_28, param_29, param_30);
                                                            _t113 = _1582;
                                                        }
                                                        else
                                                        {
                                                            if (buffer.u_warpStyleVert == 14)
                                                            {
                                                                float2 param_31 = _t113;
                                                                float param_32 = buffer.u_bend;
                                                                float2 param_33 = float2(1.0);
                                                                int param_34 = buffer.u_axisVert;
                                                                float2 _1597 = _f12(param_31, param_32, param_33, param_34);
                                                                _t113 = _1597;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out.gl_Position = float4(_t113, 0.0, 1.0);
    out.v_uv = in.a_texcoord0;
    out.gl_Position.z = (out.gl_Position.z + out.gl_Position.w) * 0.5;       // Adjust clip-space for Metal
    return out;
}

