
uniform int u_warpStyleVert;
uniform float u_bend;
uniform int u_axisVert;

attribute vec2 a_position;
varying vec2 v_uv;
attribute vec2 a_texcoord0;

float _f0(float _p0)
{
    return (_p0 * 0.5) + 0.5;
}

vec2 _f1(vec2 _p0)
{
    float param = _p0.x;
    float param_1 = _p0.y;
    return vec2(_f0(param), _f0(param_1));
}

float _f2(float _p0)
{
    return _p0 * _p0;
}

vec2 _f4(vec2 _p0, inout float _p1, int _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    vec2 _t65 = _p0;
    vec2 param = _p0;
    vec2 _t66 = _f1(param);
    float _705 = _p1;
    float _709 = (1.0 / clamp(abs(_705), 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
    float _t68 = 1.0;
    float param_1 = 1.0;
    float param_2 = _709;
    float _719 = sqrt(_f2(param_1) + _f2(param_2));
    if (_p2 == 0)
    {
        _t68 = (-cos(((2.0 * _t66.x) * atan(_t68, _709)) + atan(_709, _t68))) * _719;
        float param_3 = _719;
        float param_4 = _t68;
        _t65.x = _t68;
        _t65.y += ((sqrt(_f2(param_3) - _f2(param_4)) - _709) * sign(_p1));
    }
    else
    {
        _p1 = -_p1;
        _t68 = (-cos(((2.0 * _t66.y) * atan(_t68, _709)) + atan(_709, _t68))) * _719;
        float param_5 = _719;
        float param_6 = _t68;
        _t65.y = _t68;
        _t65.x += ((sqrt(_f2(param_5) - _f2(param_6)) - _709) * sign(_p1));
    }
    return _t65;
}

vec2 _f5(vec2 _p0, float _p1, int _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    vec2 _t72 = _p0;
    vec2 param = _p0;
    vec2 _t73 = _f1(param);
    float _820 = (1.0 / clamp(abs(_p1), 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
    float _t75 = 1.0;
    float param_1 = 1.0;
    float param_2 = _820;
    float _830 = sqrt(_f2(param_1) + _f2(param_2));
    if (_p2 == 0)
    {
        _t75 = (-cos(((2.0 * _t73.x) * atan(_t75, _820)) + atan(_820, _t75))) * _830;
        float param_3 = _830;
        float param_4 = _t75;
        _t72.x = _t75;
        _t72.y += (((sqrt(_f2(param_3) - _f2(param_4)) - _820) * sign(_p1)) * _t72.y);
    }
    else
    {
        _t75 = (-cos(((2.0 * _t73.y) * atan(_t75, _820)) + atan(_820, _t75))) * _830;
        float param_5 = _830;
        float param_6 = _t75;
        _t72.y = _t75;
        _t72.x += (((sqrt(_f2(param_5) - _f2(param_6)) - _820) * sign(_p1)) * _t72.x);
    }
    return _t72;
}

vec2 _f3(vec2 _p0, float _p1, int _p2)
{
    vec2 _t0 = _p0;
    if (_p1 > 0.0)
    {
        if (_p2 == 0)
        {
            vec2 param = _t0;
            vec2 _t1 = _f1(param);
            vec2 _130 = vec2(_t0.x, -1.0);
            float _137 = (1.0 / clamp(_p1, 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
            float _143 = (smoothstep(0.0, 1.0, _p1) * 2.0) + 1.0;
            float _t4 = _143;
            float param_1 = _143;
            float param_2 = _137;
            float _152 = sqrt(_f2(param_1) + _f2(param_2));
            _t4 = (-cos(((2.0 * _t1.x) * atan(_t4, _137)) + atan(_137, _t4))) * _152;
            float param_3 = _152;
            float param_4 = _t4;
            float _190 = ((sqrt(_f2(param_3) - _f2(param_4)) - _137) + (_p1 + 2.0)) - (_152 - _137);
            vec2 _197 = vec2(_t4, _190 - 1.0);
            vec2 _207 = normalize(_197 - _130) * ((_190 * _p1) + 2.0);
            _t0 = ((_130 + (vec2(0.0, 2.0) * _t1.y)) + ((((((_130 * (-3.0)) - vec2(0.0, 4.0)) - _207) + (_197 * 3.0)) * _t1.y) * _t1.y)) + (((((((_130 * 2.0) + vec2(0.0, 2.0)) + _207) - (_197 * 2.0)) * _t1.y) * _t1.y) * _t1.y);
        }
        else
        {
            vec2 param_5 = _t0;
            vec2 _t17 = _f1(param_5);
            vec2 _270 = vec2(-1.0, _t0.y);
            float _275 = (1.0 / clamp(_p1, 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
            float _280 = (smoothstep(0.0, 1.0, _p1) * 2.0) + 1.0;
            float _t20 = _280;
            float param_6 = _280;
            float param_7 = _275;
            float _289 = sqrt(_f2(param_6) + _f2(param_7));
            _t20 = (-cos(((2.0 * _t17.y) * atan(_t20, _275)) + atan(_275, _t20))) * _289;
            float param_8 = _289;
            float param_9 = _t20;
            float _327 = ((sqrt(_f2(param_8) - _f2(param_9)) - _275) + (_p1 + 2.0)) - (_289 - _275);
            vec2 _334 = vec2(_327 - 1.0, _t20);
            vec2 _344 = normalize(_334 - _270) * ((_327 * _p1) + 2.0);
            _t0 = -(((_270 + (vec2(2.0, 0.0) * _t17.x)) + ((((((_270 * (-3.0)) - vec2(4.0, 0.0)) - _344) + (_334 * 3.0)) * _t17.x) * _t17.x)) + (((((((_270 * 2.0) + vec2(2.0, 0.0)) + _344) - (_334 * 2.0)) * _t17.x) * _t17.x) * _t17.x));
        }
    }
    else
    {
        if (_p2 == 0)
        {
            vec2 param_10 = _t0;
            vec2 _t33 = _f1(param_10);
            vec2 _410 = vec2(_t0.x, -1.0);
            float _416 = (1.0 / clamp(abs(_p1), 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
            float _t36 = 1.0;
            float param_11 = 1.0;
            float param_12 = _416;
            float _426 = sqrt(_f2(param_11) + _f2(param_12));
            _t36 = (-cos(((2.0 * _t33.x) * atan(_t36, _416)) + atan(_416, _t36))) * _426;
            float param_13 = _426;
            float param_14 = _t36;
            float _452 = sqrt(_f2(param_13) - _f2(param_14)) - _416;
            vec2 _467 = vec2(_t36, 1.0 - _452);
            vec2 _t43 = vec2(_t36, _452) - vec2(0.0, -_416);
            _t43.y = _t43.y;
            vec2 _479 = _t43;
            vec2 _491 = mix(normalize(_479) * ((-pow(_452, 0.5)) + 2.0), vec2(0.0, -2.0), vec2(1.0 - abs(_p1)));
            _t43 = _491;
            _t0 = ((_467 + (_491 * _t33.y)) + ((((((_467 * (-3.0)) - (_491 * 2.0)) - vec2(0.0, -2.0)) + (_410 * 3.0)) * _t33.y) * _t33.y)) + (((((((_467 * 2.0) + _491) + vec2(0.0, -2.0)) - (_410 * 2.0)) * _t33.y) * _t33.y) * _t33.y);
        }
        else
        {
            vec2 param_15 = _t0;
            vec2 _t49 = _f1(param_15);
            vec2 _552 = vec2(-1.0, _t0.y);
            float _558 = (1.0 / clamp(abs(_p1), 9.9999997473787516355514526367188e-05, 1.0)) - 1.0;
            float _t52 = 1.0;
            float param_16 = 1.0;
            float param_17 = _558;
            float _568 = sqrt(_f2(param_16) + _f2(param_17));
            _t52 = (-cos(((2.0 * _t49.y) * atan(_t52, _558)) + atan(_558, _t52))) * _568;
            float param_18 = _568;
            float param_19 = _t52;
            float _594 = sqrt(_f2(param_18) - _f2(param_19)) - _558;
            vec2 _608 = vec2(1.0 - _594, _t52);
            vec2 _t59 = vec2(_t52, _594) - vec2(-_558, 0.0);
            _t59.x = _t59.x;
            vec2 _620 = _t59;
            vec2 _634 = mix(normalize(_620) * ((-pow(_t52, 0.5)) + 2.0), vec2(-2.0, -0.0), vec2(1.0 - abs(_p1)));
            _t59 = _634;
            _t0 = -(((_608 + (_634 * _t49.x)) + ((((((_608 * (-3.0)) - (_634 * 2.0)) - vec2(-2.0, 0.0)) + (_552 * 3.0)) * _t49.x) * _t49.x)) + (((((((_608 * 2.0) + _634) + vec2(-2.0, 0.0)) - (_552 * 2.0)) * _t49.x) * _t49.x) * _t49.x));
        }
    }
    return _t0;
}

vec2 _f6(vec2 _p0, float _p1, int _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    vec2 _t79 = _p0;
    vec2 param = _p0;
    vec2 _t80 = _f1(param);
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

vec2 _f7(vec2 _p0, float _p1, int _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    vec2 _t83 = _p0;
    vec2 param = _p0;
    vec2 _t84 = _f1(param);
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

vec2 _f8(vec2 _p0, inout float _p1, int _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    vec2 _t87 = _p0;
    vec2 param = _p0;
    vec2 _t88 = _f1(param);
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

vec2 _f9(vec2 _p0, float _p1, int _p2)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    vec2 _t91 = _p0;
    vec2 param = _p0;
    vec2 _t92 = _f1(param);
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

vec2 _f13(vec2 _p0, float _p1)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    vec2 _t103 = _p0;
    vec2 param = _p0;
    vec2 _t104 = _f1(param);
    float _1291 = sin(_t104.x * 3.141592502593994140625) * sin(_t104.y * 3.141592502593994140625);
    float _t106 = mix(1.0, 0.25, abs(_p1) * _1291);
    if (_p1 > 0.0)
    {
        _t106 = mix(1.0, 1.89999997615814208984375, abs(_p1) * _1291);
    }
    vec2 _1309 = _t103;
    vec2 _1311 = _1309 * _t106;
    _t103 = _1311;
    return _1311;
}

vec2 _f14(vec2 _p0, float _p1)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0;
    }
    vec2 _t107 = _p0;
    vec2 param = _p0;
    float _t109 = (1.0 / clamp(abs(_p1 * _t107.y), 9.9999997473787516355514526367188e-05, 1.0)) - 0.100000001490116119384765625;
    float param_1 = 1.0;
    float param_2 = _t109;
    float param_3 = sqrt(_f2(param_1) + _f2(param_2));
    float param_4 = _t107.x;
    float _1358 = _t109;
    _t109 = (1.0 / clamp(abs(_p1 * _t107.x), 9.9999997473787516355514526367188e-05, 1.0)) - 0.100000001490116119384765625;
    float param_5 = 1.0;
    float param_6 = _t109;
    float param_7 = sqrt(_f2(param_5) + _f2(param_6));
    float _1380 = _t107.y;
    float _1382 = _t107.y;
    _t107.y += (((sqrt(_f2(param_3) - _f2(param_4)) - _1358) * sign(_t107.y)) * sign(_p1));
    _t107.x += (((sqrt(_f2(param_7) - (_1380 * _1382)) - _t109) * sign(_t107.x)) * sign(_p1));
    return _t107;
}

vec2 _f10(vec2 _p0, float _p1)
{
    float _1137 = (_p1 / 180.0) * 3.141592502593994140625;
    float _1142 = cos(_1137);
    float _1144 = sin(_1137);
    return mat2(vec2(_1142, _1144), vec2(-_1144, _1142)) * _p0;
}

vec2 _f11(vec2 _p0, inout float _p1, int _p2)
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
    vec2 _t97 = _p0;
    vec2 param = _p0;
    vec2 _t98 = _f1(param);
    float _1188 = sin(_t98.x * 3.141592502593994140625) * sin(_t98.y * 3.141592502593994140625);
    vec2 _1196 = _t97;
    vec2 _1198 = _1196 * mix(1.0, 0.75, abs(_p1) * _1188);
    _t97 = _1198;
    vec2 param_1 = _1198;
    float param_2 = (_p1 * _1188) * 45.0;
    vec2 _1207 = _f10(param_1, param_2);
    _t97 = _1207;
    return _1207;
}

vec2 _f12(vec2 _p0, inout float _p1, vec2 _p2, int _p3)
{
    if (abs(_p1) <= 9.9999997473787516355514526367188e-05)
    {
        return _p0 * _p2;
    }
    vec2 _1223 = _p0 * _p2;
    vec2 _t101 = _1223;
    vec2 param = _1223;
    vec2 _1231 = (_f1(param) - vec2(0.5)) * 2.0;
    vec2 _t102 = (vec2(1.0) - (_1231 * _1231)) * 0.5;
    if (_p3 != 0)
    {
        _p1 = -_p1;
    }
    _t101.y += ((_t102.x * _p1) * _t101.y);
    _t101.x -= ((_t102.y * _p1) * _t101.x);
    return _t101;
}

void main()
{
    vec2 _t113 = a_position;
    if (u_warpStyleVert == 0)
    {
    }
    else
    {
        if (u_warpStyleVert == 1)
        {
        }
        else
        {
            if (u_warpStyleVert == 2)
            {
            }
            else
            {
                if (u_warpStyleVert == 3)
                {
                    vec2 param = _t113;
                    float param_1 = u_bend;
                    int param_2 = u_axisVert;
                    vec2 _1455 = _f4(param, param_1, param_2);
                    _t113 = _1455;
                }
                else
                {
                    if (u_warpStyleVert == 4)
                    {
                        vec2 param_3 = _t113;
                        float param_4 = u_bend;
                        int param_5 = u_axisVert;
                        _t113 = _f5(param_3, param_4, param_5);
                    }
                    else
                    {
                        if (u_warpStyleVert == 5)
                        {
                            vec2 param_6 = _t113;
                            float param_7 = u_bend;
                            int param_8 = u_axisVert;
                            _t113 = -_f3(param_6, param_7, param_8);
                        }
                        else
                        {
                            if (u_warpStyleVert == 6)
                            {
                                vec2 param_9 = _t113;
                                float param_10 = u_bend;
                                int param_11 = u_axisVert;
                                _t113 = _f3(param_9, param_10, param_11);
                            }
                            else
                            {
                                if (u_warpStyleVert == 7)
                                {
                                    vec2 param_12 = _t113;
                                    float param_13 = u_bend;
                                    int param_14 = u_axisVert;
                                    _t113 = _f6(param_12, param_13, param_14);
                                }
                                else
                                {
                                    if (u_warpStyleVert == 8)
                                    {
                                        vec2 param_15 = _t113;
                                        float param_16 = u_bend;
                                        int param_17 = u_axisVert;
                                        _t113 = _f7(param_15, param_16, param_17);
                                    }
                                    else
                                    {
                                        if (u_warpStyleVert == 9)
                                        {
                                            vec2 param_18 = _t113;
                                            float param_19 = u_bend;
                                            int param_20 = u_axisVert;
                                            vec2 _1534 = _f8(param_18, param_19, param_20);
                                            _t113 = _1534;
                                        }
                                        else
                                        {
                                            if (u_warpStyleVert == 10)
                                            {
                                                vec2 param_21 = _t113;
                                                float param_22 = u_bend;
                                                int param_23 = u_axisVert;
                                                _t113 = _f9(param_21, param_22, param_23);
                                            }
                                            else
                                            {
                                                if (u_warpStyleVert == 11)
                                                {
                                                    vec2 param_24 = _t113;
                                                    float param_25 = u_bend;
                                                    _t113 = _f13(param_24, param_25);
                                                }
                                                else
                                                {
                                                    if (u_warpStyleVert == 12)
                                                    {
                                                        vec2 param_26 = _t113;
                                                        float param_27 = u_bend;
                                                        _t113 = _f14(param_26, param_27);
                                                    }
                                                    else
                                                    {
                                                        if (u_warpStyleVert == 13)
                                                        {
                                                            vec2 param_28 = _t113;
                                                            float param_29 = u_bend;
                                                            int param_30 = u_axisVert;
                                                            vec2 _1582 = _f11(param_28, param_29, param_30);
                                                            _t113 = _1582;
                                                        }
                                                        else
                                                        {
                                                            if (u_warpStyleVert == 14)
                                                            {
                                                                vec2 param_31 = _t113;
                                                                float param_32 = u_bend;
                                                                vec2 param_33 = vec2(1.0);
                                                                int param_34 = u_axisVert;
                                                                vec2 _1597 = _f12(param_31, param_32, param_33, param_34);
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
    gl_Position = vec4(_t113, 0.0, 1.0);
    v_uv = a_texcoord0;
}

