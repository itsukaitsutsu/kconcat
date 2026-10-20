local exports = exports or {}
local TempScriptLua = TempScriptLua or {}
TempScriptLua.__index = TempScriptLua
---@class TempScriptLua : ScriptComponent
---@field FilterIntensity number [UI(Slider,Range={0,1})]
---
function TempScriptLua.new(construct, ...)
    local self = setmetatable({}, TempScriptLua)
    self.FilterIntensity = 1.0
    if construct and TempScriptLua.constructor then TempScriptLua.constructor(self, ...) end
    return self
end

function TempScriptLua:constructor()
    self.name = "scriptComp"
end

local function remap(value, srcMin, srcMax, dstMin, dstMax)
    return dstMin + (value - srcMin) * (dstMax - dstMin) / (srcMax - srcMin)
end

local function getXYScale(width, height)
    --[[
        Description: get XY scale for adjusting width/height ratio.
    ]]
    -- the following computation for baseSize is to avoid too small or too large width/height ratio.
    local size1 = math.min(width, height)
    local size2 = math.max(width, height) / 2.
    local baseSize = math.max(size1, size2)

    local xScale = baseSize / width
    local yScale = baseSize / height
    return xScale, yScale
end

local function print(...)
    -- local arg = { ... }
    -- local msg = "effect_lua:"
    -- for k, v in pairs(arg) do
    --     msg = msg .. tostring(v) .. " "
    -- end
    -- Amaz.LOGI("dyp_sticker", msg)
end

local function parseMeshData(data)
    local vertices = {}
    local indices = {}
    local index = 1

    for line in data:gmatch("[^\r\n]+") do
        local parts = {}
        for part in line:gmatch("%S+") do
            table.insert(parts, tonumber(part))
        end

        if #parts == 3 then
            table.insert(indices, parts)
        else
            table.insert(vertices, parts)
        end
    end

    return vertices, indices
end

local meshData = 
[[
0.083288535475731 0.586141705513
0.057340525090694 0.54092609882355
0.05707349255681 0.49139004945755
0.080487653613091 0.44294944405556
0.12655130028725 0.40077066421509
0.19087389111519 0.36630314588547
0.26355183124542 0.33810994029045
0.34394225478172 0.31970879435539
0.43177020549774 0.31003034114838
0.52442222833633 0.31765812635422
0.61116099357605 0.336291462183
0.68815588951111 0.36289983987808
0.75615173578262 0.3965058028698
0.80629456043243 0.43814516067505
0.8324681520462 0.48673534393311
0.83287119865417 0.5381960272789
0.80525690317154 0.58509814739227
0.75190317630768 0.61413264274597
0.68868309259415 0.63893413543701
0.61971670389175 0.65855574607849
0.53214430809021 0.67242580652237
0.4335809648037 0.67715340852737
0.32854297757149 0.6710889339447
0.2351152151823 0.6542963385582
0.17824921011925 0.6354176402092
0.12796381115913 0.61156821250916
0.060707550495863 0.62631636857986
0.033093873411417 0.66418588161469
0.0056046987883747 0.69927155971527
-0.014473765157163 0.73760259151459
0.05544314160943 0.72354590892792
0.12048368901014 0.70392972230911
0.1824302226305 0.68137145042419
0.6686235666275 0.68970602750778
0.72581797838211 0.71756118535995
0.7873221039772 0.74084013700485
0.85884338617325 0.75327944755554
0.86453074216843 0.71265441179276
0.84869986772537 0.67104011774063
0.82923829555511 0.62789213657379
0.43363964557648 0.61283880472183
0.43209752440453 0.54763722419739
0.43094605207443 0.48348751664162
0.43018153309822 0.43901008367538
0.4293629527092 0.39987477660179
0.37344101071358 0.39348804950714
0.48491099476814 0.39278918504715
0.42996230721474 0.3669852912426
0.43025013804436 0.35345733165741
0.41611671447754 0.3481941819191
0.40014871954918 0.34487429261208
0.38292166590691 0.34271937608719
0.36489644646645 0.34167343378067
0.38281574845314 0.34241038560867
0.39994806051254 0.3445051908493
0.41585505008698 0.34783652424812
0.42999637126923 0.35311564803123
0.44375690817833 0.34770336747169
0.45995864272118 0.34399002790451
0.47713294625282 0.34157860279083
0.49503621459007 0.34055486321449
0.47709792852402 0.34185764193535
0.45997515320778 0.34436041116714
0.44394147396088 0.348064661026
0.21100203692913 0.50443875789642
0.25024032592773 0.51479852199554
0.2975837290287 0.51086407899857
0.33086833357811 0.49247443675995
0.33470103144646 0.46836668252945
0.29279151558876 0.45694389939308
0.24440030753613 0.46070748567581
0.21413843333721 0.47996562719345
0.65544778108597 0.49886226654053
0.61745369434357 0.51005417108536
0.56944930553436 0.50815814733505
0.53404659032822 0.49136003851891
0.52885794639587 0.46718400716782
0.56967836618423 0.45346239209175
0.61926525831223 0.45524504780769
0.65177398920059 0.47382912039757
0.27184385061264 0.48634859919548
0.59395551681519 0.48216927051544
37 36 35
35 36 29
21 34 35
27 26 31
72 12 13
72 15 16
16 15 39
30 35 29
30 27 31
30 31 21
21 35 30
71 4 5
3 4 71
26 27 1
11 12 79
12 72 79
14 72 13
15 72 14
37 15 14
37 35 38
35 39 38
38 15 37
38 39 15
28 30 29
27 30 28
28 1 27
29 2 28
2 1 28
5 6 70
70 71 5
65 71 70
64 2 3
3 71 64
64 1 2
64 71 65
78 10 11
11 79 78
22 40 21
21 31 22
31 32 22
17 72 16
17 18 72
16 39 17
34 18 17
17 35 34
17 39 35
52 45 6
6 7 52
6 45 69
69 70 6
24 64 65
24 32 31
10 78 77
21 40 20
20 34 21
33 18 34
33 19 18
34 20 33
33 20 19
53 52 7
53 7 8
8 54 53
50 54 55
55 49 50
55 54 8
65 70 80
70 69 80
23 22 32
32 24 23
23 24 65
64 24 25
31 26 25
25 24 31
9 10 46
10 77 46
73 79 72
73 78 79
72 18 73
18 19 73
45 52 51
52 53 51
50 45 51
51 54 50
51 53 54
63 48 57
57 55 8
49 55 56
56 57 48
55 57 56
1 64 0
64 25 0
26 1 0
0 25 26
9 46 60
47 48 63
47 45 50
50 49 47
47 56 48
49 56 47
76 46 77
41 67 42
81 77 78
78 73 81
19 20 74
74 73 19
74 81 73
40 41 74
74 20 40
77 81 74
66 80 69
69 67 66
65 80 66
66 23 65
22 23 66
66 67 41
66 41 40
40 22 66
68 69 45
68 67 69
68 42 67
61 60 46
46 76 43
76 42 43
43 68 45
42 68 43
75 76 77
77 74 75
75 74 41
41 42 75
75 42 76
62 61 46
62 47 63
46 47 62
9 60 59
60 61 59
8 9 59
44 47 46
46 43 44
45 47 44
44 43 45
61 62 58
58 59 61
8 59 58
58 57 8
63 57 58
58 62 63
]]

local dogMeshData = [[
0.28060778975487 0.83103567361832
0.26684600114822 0.77963972091675
0.26671054959297 0.72778588533401
0.28439718484879 0.67915832996368
0.31166049838066 0.63404542207718
0.33180671930313 0.60133004188538
0.36892375349998 0.57509464025497
0.42310538887978 0.55953878164291
0.48500311374664 0.55079305171967
0.55011999607086 0.54949462413788
0.6092649102211 0.55619949102402
0.65540164709091 0.57507705688477
0.68821102380753 0.60300010442734
0.7356094121933 0.64070338010788
0.77652180194855 0.6818859577179
0.80547279119492 0.72911757230759
0.8211522102356 0.77881270647049
0.81645303964615 0.8061695098877
0.80649077892303 0.832495033741
0.78900933265686 0.85954642295837
0.6997361779213 0.89471393823624
0.58386451005936 0.91434121131897
0.46349504590034 0.91758269071579
0.35713717341423 0.90118414163589
0.32581341266632 0.8788423538208
0.30154353380203 0.85594832897186
0.56910806894302 0.86095482110977
0.55370777845383 0.80499655008316
0.53816998004913 0.74913781881332
0.53111666440964 0.71796554327011
0.52469539642334 0.68932223320007
0.51861089468002 0.66087138652802
0.47063404321671 0.65973085165024
0.44608187675476 0.63785403966904
0.46551457047462 0.61596441268921
0.50331449508667 0.60529381036758
0.54449284076691 0.60903769731522
0.57211434841156 0.62600791454315
0.56399637460709 0.65116459131241
0.47824156284332 0.63377231359482
0.51111543178558 0.6304252743721
0.54373496770859 0.62792241573334
0.49517753720284 0.5809788107872
0.46826013922691 0.58067911863327
0.44114419817924 0.58062869310379
0.41438204050064 0.582534968853
0.3913526237011 0.58733010292053
0.41531229019165 0.58216285705566
0.44097658991814 0.58000695705414
0.46786937117577 0.57967275381088
0.4944466650486 0.57964652776718
0.52114778757095 0.57545787096024
0.54676747322083 0.57168585062027
0.57180064916611 0.56971341371536
0.59603244066238 0.57109677791595
0.57284617424011 0.57001268863678
0.54721128940582 0.57229089736938
0.52170938253403 0.57648187875748
0.3379565179348 0.76767057180405
0.36264780163765 0.77779895067215
0.39540097117424 0.7793442606926
0.41973289847374 0.7673214673996
0.42399361729622 0.74990838766098
0.39880755543709 0.74181324243546
0.36843898892403 0.74247044324875
0.34623703360558 0.75255256891251
0.73012053966522 0.73171979188919
0.71402704715729 0.74550521373749
0.68424552679062 0.75247782468796
0.65585416555405 0.744853079319
0.64225476980209 0.72922086715698
0.66195976734161 0.71723890304565
0.69036561250687 0.71288990974426
0.71583300828934 0.71899217367172
0.38182103633881 0.76129710674286
0.68780744075775 0.73291236162186
0.18919743597507 0.84383338689804
0.11590261012316 0.82444602251053
0.12538850307465 0.76707184314728
0.16332863271236 0.71488469839096
0.19262656569481 0.76507812738419
0.21812583506107 0.82239818572998
0.26383477449417 0.87720435857773
0.86286509037018 0.81953448057175
0.87676274776459 0.75988525152206
0.87232667207718 0.70169442892075
0.87685805559158 0.65095853805542
0.932013630867 0.69399297237396
0.9645339846611 0.74589520692825
0.91078108549118 0.77660858631134
68 20 69
70 69 27
79 6 5
17 83 18
18 83 19
13 86 14
86 87 85
85 14 86
15 14 85
16 83 17
68 69 75
2 79 3
80 78 79
79 2 80
80 2 1
77 78 80
13 72 12
12 86 13
12 11 86
86 11 10
23 60 22
1 2 58
11 12 37
26 69 20
26 27 69
20 21 26
26 21 22
22 27 26
22 60 61
61 27 22
84 85 87
83 16 84
15 85 84
84 16 15
66 14 15
15 16 66
66 16 17
81 80 1
60 23 24
24 82 25
24 23 82
0 58 25
1 58 0
0 81 1
25 82 0
82 81 0
62 61 63
63 61 60
71 37 12
71 12 72
72 75 71
71 69 70
71 75 69
54 10 11
11 37 54
70 27 28
28 29 70
62 29 28
28 61 62
27 61 28
46 5 6
6 7 46
88 83 89
83 84 89
89 87 88
89 84 87
13 14 73
14 66 73
73 72 13
73 75 72
17 18 67
67 66 17
67 18 19
67 20 68
67 19 20
68 75 67
75 73 67
67 73 66
77 80 76
80 81 76
76 82 77
76 81 82
60 24 59
25 58 59
59 24 25
5 46 4
4 46 33
3 79 4
79 5 4
41 36 37
55 54 37
37 36 55
10 54 55
31 30 32
62 63 32
32 63 33
32 29 62
32 30 29
56 36 57
57 52 56
7 8 48
3 4 64
33 63 64
64 4 33
38 41 37
38 30 31
31 41 38
38 71 70
37 71 38
70 29 38
29 30 38
53 36 56
53 55 36
56 52 53
53 52 9
9 10 53
10 55 53
33 46 45
45 46 7
65 59 58
65 64 59
3 64 65
65 2 3
65 58 2
60 59 74
59 64 74
74 63 60
74 64 63
57 42 51
51 8 9
9 52 51
52 57 51
33 44 34
35 57 36
35 42 57
47 48 44
47 44 33
33 45 47
7 48 47
47 45 7
8 51 50
50 51 42
34 35 39
39 32 33
33 34 39
31 32 39
43 50 42
42 35 43
43 35 34
43 34 44
31 39 40
40 39 35
40 41 31
36 41 40
40 35 36
8 50 49
50 43 49
49 48 8
44 48 49
49 43 44
]]

function TempScriptLua:parseMesh(data)
    local vertices, indices = parseMeshData(data)
    self.vertices = vertices
    self.indices = indices
    -- print("Vertices:")
    -- for i, vertex in ipairs(vertices) do
    --     print(string.format("Vertex %d: %s", i, table.concat(vertex, ", ")))
    -- end

    -- print("\nIndices:")
    -- for i, index in ipairs(indices) do
    --     print(string.format("Index %d: %s", i, table.concat(index, ", ")))
    -- end
end

function TempScriptLua:onStart(comp)
    self.MaxGaussianBlurSample = 30.
    self.NormalizationSize = 1000.
    self.RadiusOverSigma = 2.5
    self.GaussianBlurScale = 2.0
    self.skinMapBlurIntensity = 0.1
    self.matGaussianBlurX = comp.entity.scene:findEntityBy("EntityBlurX"):getComponent("MeshRenderer").material
    self.matGaussianBlurY = comp.entity.scene:findEntityBy("EntityBlurY"):getComponent("MeshRenderer").material
    self.FilterMaterial = comp.entity.scene:findEntityBy("SkinSegFilter"):getComponent("MeshRenderer").material
    self.DehazeMaterial = comp.entity.scene:findEntityBy("EntityDehaze"):getComponent("MeshRenderer").material
    self.meshRenderer = comp.entity.scene:findEntityBy("pet"):getComponent("MeshRenderer")
    self.DehazeMaterial:setFloat("intensity", 0.0)
    self.OutputTex =  comp.entity.scene:getOutputRenderTexture()
    self.OutputTex.attachment = Amaz.RenderTextureAttachment.NONE
end

function TempScriptLua:onUpdate(comp, deltaTime)
    if self.OutputTex then
        local textureWidth = self.OutputTex.width
        local textureHeight = self.OutputTex.height
        local xScale, yScale = getXYScale(textureWidth, textureHeight)
        local dx = xScale / self.NormalizationSize
        local dy = yScale / self.NormalizationSize
        local sample = remap(self.skinMapBlurIntensity, 0., 1., 0., self.MaxGaussianBlurSample)
        local sigmaX = sample * dx / self.RadiusOverSigma
        local sigmaY = sample * dy / self.RadiusOverSigma
        self.matGaussianBlurX:setFloat("u_sampleX", sample)
        self.matGaussianBlurX:setFloat("u_sigmaX", sigmaX * self.GaussianBlurScale)
        self.matGaussianBlurX:setFloat("u_stepX", dx * self.GaussianBlurScale)
        self.matGaussianBlurY:setFloat("u_sampleY", sample)
        self.matGaussianBlurY:setFloat("u_sigmaY", sigmaY * self.GaussianBlurScale)
        self.matGaussianBlurY:setFloat("u_stepY", dy * self.GaussianBlurScale)
    
    end
    if self.meshRenderer then
        print('running: meshRenderer is not nil')

        local material = self.meshRenderer.material

        local mesh = self.meshRenderer.mesh
        local submeshes = mesh.submeshes
        submeshes:clear()


        local result = Amaz.Algorithm.getAEAlgorithmResult()
        local facePetCount = result:getFacePetInfoCount()

        if facePetCount > 0 then
            print("facePet count " .. facePetCount)
            self.meshRenderer.enabled = true
            
            local facePetInfo = result:getFacePetInfo(0)
            if facePetInfo then
                local points = facePetInfo.points_array
                local face_pet_type = facePetInfo.face_pet_type -- 1 cat 2 dog
                material:setFloat("u_face_pet_type", face_pet_type)
                if face_pet_type==1 then
                    self:parseMesh(meshData) -- todo once
                elseif face_pet_type ==2 then                
                    self:parseMesh(dogMeshData)
                else
                    print("facePet type is not cat or dog")
                    return
                end
                print("facePet type ".. face_pet_type)
                local size = points:size() -8
                local vertex = Amaz.Vec3Vector()
                local uvArray = Amaz.Vec2Vector()
                local indices = Amaz.UInt16Vector()

                local submesh = Amaz.SubMesh()
                print("facePet point size " .. size)

                -- for i = 1,points:size() do
                --     print("facePet point ".. i .. " ".. points:get(i - 1).x.. " ".. points:get(i - 1).y)
                -- end
                for i = 1, #self.vertices do
                    local point = points:get(i - 1)
                    vertex:pushBack(Amaz.Vector3f(point.x, point.y, 0))
                    local uv = self.vertices[i]
                    uvArray:pushBack(Amaz.Vector2f(uv[1], uv[2]))
                end

                for i = 1, #self.indices do
                    indices:pushBack(self.indices[i][1])
                    indices:pushBack(self.indices[i][2])
                    indices:pushBack(self.indices[i][3])
                end

                mesh:setVertexArray(vertex)
                mesh:setUvArray(0, uvArray)

                print("facePet vertex size " .. vertex:size())
                print("facePet uv size " .. uvArray:size())
                print("facePet indices size " .. indices:size())

                submesh.indices16 = indices
                submesh.mesh = mesh
                submeshes:pushBack(submesh)

                mesh.submeshes = submeshes
            else
                self.meshRenderer.enabled = false
                print("facePet info index 0 is nil")
            end
        else
            print("facePet count is 0")
            self.meshRenderer.enabled = false
        end
    end
end

function TempScriptLua:onEvent(sys, event)
    if "desharpe_intensity" == event.args:get(0) then
        local intensity = event.args:get(1)
        self.FilterMaterial:setFloat("intensity", intensity)
        self.DehazeMaterial:setFloat("intensity", intensity*0.1)
    end
end

exports.TempScriptLua = TempScriptLua
return exports
