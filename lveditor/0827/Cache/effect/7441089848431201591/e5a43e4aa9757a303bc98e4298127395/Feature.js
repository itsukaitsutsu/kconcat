const Amaz = effect.Amaz;

class Feature
{
    onInit()
    {
        // this.scene = null;
        this.scenesMap = {};
        // this.material = null;
        this.colorMigrationMaterial = null;
        this.lutTexture = null;
        this.targetPath = "";
        this.sourcePath = "";
        this.targetUseCache = 1;
        this.sourceUseCache = 1; 
        this.isFirstFrame = true;
        this.m_maskPreviewColor = new Amaz.Vector4f(0.0, 0.0, 0.0, 1.0);
        this.m_maskBlendMode = 0;
        this.setParamsDirty = false;
        this.hasPostEffect = false;
        // haha, add a trick logic
        this.neverHasPostEffectFlag = true;

        this.sceneIntensityKeyMap = {
            "AmazingFeature_adjustColor": ["temperature_intensity", "tint_intensity", 
            "saturation_intensity", "brightness_intensity", "contrast_intensity", 
            "highlight_intensity", "shadow_intensity", "white_intensity", 
            "black_intensity", "light_sensation_intensity", "fade_intensity"],
            "AmazingFilter_lut": ["lut_intensity"],
            "AmazingFeature_clear": ["clear_intensity"],
            "AmazingFeature_colorCorrection": ["colorcorrection_intensity"],
            "AmazingFeature_colorMigration": ["colormigration_intensity"],
            "AmazingFeature_particle": ["particle_intensity"],
            "AmazingFeature_sharpen": ["sharpen_intensity"],
            "AmazingFeature_smartColorAdjustment": ["smartcoloradjustment_intensity"],
            "AmazingFeature_vignetting": ["vignetting_intensity"]
        };
        
        this.intensityKeyParamsMap = {
            "temperature_intensity": this.temperature_intensity,
            "tint_intensity": this.tint_intensity,
            "saturation_intensity": this.saturation_intensity,
            "brightness_intensity": this.brightness_intensity,
            "contrast_intensity": this.contrast_intensity,
            "highlight_intensity": this.highlight_intensity,
            "shadow_intensity": this.shadow_intensity,
            "white_intensity": this.white_intensity,
            "black_intensity": this.black_intensity,
            "light_sensation_intensity": this.light_sensation_intensity,
            "fade_intensity": this.fade_intensity,
            "clear_intensity": this.clear_intensity,
            "colorcorrection_intensity": this.colorcorrection_intensity,
            "colormigration_intensity": this.colormigration_intensity,
            "particle_intensity": this.particle_intensity,
            "sharpen_intensity": this.sharpen_intensity,
            "smartcoloradjustment_intensity": this.smartcoloradjustment_intensity,
            "vignetting_intensity": this.vignetting_intensity,
            "lut_intensity": this.lut_intensity
        };

        this.customSceneEnableMap = {
            "smartcoloradjustment_enable": false,
            "colormigration_enable": false,
            "colorcorrection_enable": false,
            "hsl_enable": false,
            "color_curves_enable": false,
            "primary_color_wheels_enable": false,
            "log_color_wheels_enable": false,
            "lut_enable": false
        }

        // clear all intensity
        for (const key of Object.keys(this.intensityKeyParamsMap)){
            this.intensityKeyParamsMap[key] = 0.0;
        }

        // disable all scenes by default
        for (let key of Object.keys(this.scenesMap)){
            this.scenesMap[key].visible = false;
        }
    }

    onLoadScenes(...scenes)
    {
        // this.scene = scenes[0];
        // var entity = this.scene.findEntityBy("ColorMigration");
        // var render = entity.getComponent("MeshRenderer");
        // this.material = render.sharedMaterial;
        for (let index = 0; index < scenes.length; index++) {
            const scene = scenes[index];
            let rootDir = scene.assetMgr.rootDir;
            let splitDir = rootDir.substring(rootDir.lastIndexOf("/") + 1);
            if (splitDir == "") {
                let splitArr = rootDir.split("/");
                splitDir = splitArr[splitArr.length - 2];
            }
            this.scenesMap[splitDir] = scene;
            // console.log('wjj test Feature.onLoadScenes, this.scenesMap[splitDir]', splitDir);
        }

        if (this.scenesMap['AmazingFeature_colorMigration'] != null)
        {
            let colorMigrationEntity = this.scenesMap['AmazingFeature_colorMigration'].findEntityBy('ColorMigration');
            let colorMigrationrender = colorMigrationEntity.getComponent('MeshRenderer');
            this.colorMigrationMaterial = colorMigrationrender.material;
        }

        if (this.scenesMap['AmazingFeature_blit'] != null && this.scenesMap['AmazingFeature_blend'] != null)
        {
            let inputRT = this.scenesMap['AmazingFeature_blit'].findEntityBy('Camera_blit').getComponent('Camera').renderTexture;
            this.scenesMap['AmazingFeature_blend'].findEntityBy('blend').getComponent('MeshRenderer').material.setTex('u_inputTex', inputRT);
            let adjustRT = this.scenesMap['AmazingFeature_blit_color'].findEntityBy('Camera_blit').getComponent('Camera').renderTexture;
            this.scenesMap['AmazingFeature_blend'].findEntityBy('blend').getComponent('MeshRenderer').material.setTex('u_adjustTex', adjustRT);
        }
    }

    onUnloadScenes()
    {
        // this.scene = null;
        // this.material = null;
        this.scenesMap = {};
        this.colorMigrationMaterial = null;
        this.lutTexture = null; 
    }

    onSetParameters(jsonStr)
    {
        this.setParamsDirty = true;
        // disable all scenes by default
        for (let key of Object.keys(this.scenesMap)){
            this.scenesMap[key].visible = false;
        }
        const jsonParam = JSON.parse(jsonStr);
        // clear all intensity
        for (const key of Object.keys(this.intensityKeyParamsMap)){
            if (key in jsonParam){
                for (const key of Object.keys(this.intensityKeyParamsMap)){
                    this.intensityKeyParamsMap[key] = 0.0;
                }
                break;
            }
        }
        if (jsonParam)
        {
            if ('blendMode' in jsonParam){
                if (jsonParam.blendMode === false)
                    this.m_maskBlendMode = 0;
                else
                    this.m_maskBlendMode = 1;
            }
            if ('previewColor' in jsonParam){
                if (jsonParam.previewColor instanceof Array)
                {
                    let color = jsonParam.previewColor;
                    this.m_maskPreviewColor = new Amaz.Vector4f(color[0], color[1], color[2], color[3]);
                }
            }
            if ('smartcoloradjustment_enable' in jsonParam){
                this.customSceneEnableMap["smartcoloradjustment_enable"] = jsonParam.smartcoloradjustment_enable;
            }
            if ('colormigration_enable' in jsonParam){
                this.customSceneEnableMap["colormigration_enable"] = jsonParam.colormigration_enable;
            }
            if ('colorcorrection_enable' in jsonParam){
                this.customSceneEnableMap["colorcorrection_enable"] = jsonParam.colorcorrection_enable;
            }
            if ('hsl_enable' in jsonParam){
                this.customSceneEnableMap["hsl_enable"] = jsonParam.hsl_enable;
            }
            if ('color_curves_enable' in jsonParam){
                this.customSceneEnableMap["color_curves_enable"] = jsonParam.color_curves_enable;
            }
            if ('primary_color_wheels_enable' in jsonParam){
                this.customSceneEnableMap["primary_color_wheels_enable"] = jsonParam.primary_color_wheels_enable;
            }
            if ('log_color_wheels_enable' in jsonParam){
                this.customSceneEnableMap["log_color_wheels_enable"] = jsonParam.log_color_wheels_enable;
            }
            if ('lut_enable' in jsonParam){
                this.customSceneEnableMap["lut_enable"] = jsonParam.lut_enable;
            }
            if ('reset_params' in jsonParam){
                if (jsonParam.reset_params == 1){
                    for (const key of Object.keys(this.customSceneEnableMap)){
                        this.customSceneEnableMap[key] = false;
                        // console.error("wjj test onSetParameters customSceneEnableMap reset_params: ", key, this.customSceneEnableMap[key]);
                    }
                }
            }
            if ('hasPostEffect' in jsonParam){
                this.neverHasPostEffectFlag = false;
                if (jsonParam.hasPostEffect){
                    this.hasPostEffect = true;
                }
                else{
                    this.hasPostEffect = false;
                }
            }
            else if (this.neverHasPostEffectFlag){
                // backup logic
                if (this.scenesMap['AmazingFeature_blend'] != null){
                    let lumiRootEntity = this.scenesMap['AmazingFeature_blend'].findEntityBy("LumiRoot");
                    if (lumiRootEntity != null){
                        const trans = lumiRootEntity.getComponent('Transform');
                        let isMaskVisible = false;
                        for (let i = 0; i < trans.children.size(); ++i) {
                            if (trans.children.get(i).entity.visible == true)
                            {
                                isMaskVisible = true;
                                break;
                            }
                        }
                        this.hasPostEffect = isMaskVisible;
                    }
                }
            }

            // should update params after set params
            this._updateParameters();
        }
        return jsonStr;
    }
   
    onSetParameter(name, value)
    {
        this.setParamsDirty = true;
        if (name == "colormigration_target_path")
        {
            if (value.length > 0 && this.targetPath != value)
            {
                this.targetPath = value;
                // this.targetUseCache = 0;
            }
        }
        if (name == "colormigration_source_path")
        {
            if (value.length > 0 && this.sourcePath != value)
            {
                if (this.sourcePath != "")
                {
                    this.sourceUseCache = 0;
                }
                this.sourcePath = value;
            }
        }
        if (name == "colormigration_intensity")
        {
            this.colorMigrationMaterial.setFloat("u_intensity", parseFloat(value));
        }

        // filter keys
        if (!Object.keys(this.intensityKeyParamsMap).includes(name)){
            this._updateParameters();
            return false;
        }

        // set visible for scenes according to parameter
        for (let key of Object.keys(this.scenesMap)){
            this.scenesMap[key].visible = false;
            if (Object.keys(this.sceneIntensityKeyMap).includes(key)){
                if (Math.abs(value) < 0.001){
                    break;
                }
                if (this.sceneIntensityKeyMap[key].includes(name)){
                    this.intensityKeyParamsMap[name] = value;
                    break;
                }
            }
        }
        // should update params after set params
        this._updateParameters();
    }
   
    onGetParameter(name)
    {
        switch(name)
        {
            case "is_skip_enable":
                return false;
        }
    }

    _updateParameters()
    {
        if (this.setParamsDirty)
        {
            // check all scenes visibal
            for (let key of Object.keys(this.scenesMap)){
                // filter keys
                if (!Object.keys(this.sceneIntensityKeyMap).includes(key)){
                    continue;
                }
                for (let k of Object.keys(this.sceneIntensityKeyMap[key])){
                    if (Math.abs(this.intensityKeyParamsMap[this.sceneIntensityKeyMap[key][k]]) > 0.001){
                        this.scenesMap[key].visible = true;
                        break;
                    }
                }
            }
            
            if (this.scenesMap['AmazingFeature_smartColorAdjustment'] != null){
                this.scenesMap['AmazingFeature_smartColorAdjustment'].visible = this.customSceneEnableMap["smartcoloradjustment_enable"];
            }
            if (this.scenesMap['AmazingFeature_colorMigration'] != null){
                this.scenesMap['AmazingFeature_colorMigration'].visible = this.customSceneEnableMap["colormigration_enable"];
            }
            if (this.scenesMap['AmazingFeature_colorCorrection'] != null){
                this.scenesMap['AmazingFeature_colorCorrection'].visible = this.customSceneEnableMap["colorcorrection_enable"];
            }
            if (this.scenesMap['AmazingFeature_hsl'] != null){
                this.scenesMap['AmazingFeature_hsl'].visible = this.customSceneEnableMap["hsl_enable"];
            }
            if (this.scenesMap['AmazingFeature_curves'] != null){
                this.scenesMap['AmazingFeature_curves'].visible = this.customSceneEnableMap["color_curves_enable"];
            }
            if (this.scenesMap['AmazingFeature_primaryWheel'] != null){
                this.scenesMap['AmazingFeature_primaryWheel'].visible = this.customSceneEnableMap["primary_color_wheels_enable"];
            }
            if (this.scenesMap['AmazingFeature_logWheel'] != null){
                this.scenesMap['AmazingFeature_logWheel'].visible = this.customSceneEnableMap["log_color_wheels_enable"];
            }
            if (this.scenesMap['AmazingFilter_lut'] != null){
                this.scenesMap['AmazingFilter_lut'].visible = this.customSceneEnableMap["lut_enable"];
            }

            // check mask visible
            if (this.hasPostEffect){
                // console.warn("featurejs hasPostEffect : true", 1);
                if (this.scenesMap['AmazingFeature_blit'] != null){
                    this.scenesMap['AmazingFeature_blit'].visible = true;
                }
                if (this.scenesMap['AmazingFeature_blit_color'] != null){
                    this.scenesMap['AmazingFeature_blit_color'].visible = true;
                }
                if (this.scenesMap['AmazingFeature_blend'] != null){
                    this.scenesMap['AmazingFeature_blend'].visible = true;
                    this.scenesMap['AmazingFeature_blend'].findEntityBy('blend').getComponent('MeshRenderer').material.setInt('u_blendWithMask', 1);
                    this.scenesMap['AmazingFeature_blend'].findEntityBy('blend').getComponent('MeshRenderer').material.setFloat("u_maskBlendMode", this.m_maskBlendMode * 1.0);
                    if (this.m_maskBlendMode === 0)
                    {
                        this.scenesMap['AmazingFeature_blend'].findEntityBy('blend').getComponent('MeshRenderer').material.setVec4("u_maskPreviewColor", new Amaz.Vector4f(0.0, 0.0, 0.0, 0.0));
                    }
                    if (this.m_maskBlendMode === 1)
                    {
                        this.scenesMap['AmazingFeature_blend'].findEntityBy('blend').getComponent('MeshRenderer').material.setVec4("u_maskPreviewColor", this.m_maskPreviewColor);
                    }
                } 
            }
            else
            {
                // console.warn("featurejs hasPostEffect : false", 0);
                if (this.scenesMap['AmazingFeature_blit'] != null){
                    this.scenesMap['AmazingFeature_blit'].visible = false;
                }
                if (this.scenesMap['AmazingFeature_blit_color'] != null){
                    this.scenesMap['AmazingFeature_blit_color'].visible = false;
                }
                if (this.scenesMap['AmazingFeature_blend'] != null){
                    this.scenesMap['AmazingFeature_blend'].findEntityBy('blend').getComponent('MeshRenderer').material.setInt('u_blendWithMask', 0);
                    this.scenesMap['AmazingFeature_blend'].visible = false;
                }
            }
            this.setParamsDirty = false;
        }
    }
   
    onBeforeAlgorithmUpdate(graphName)
    {
        var algorithm = Amaz.AmazingManager.getSingleton("Algorithm")
        if (graphName == 'colormigration_hauisdhfiuwfhaui') {
            if (this.scenesMap['AmazingFeature_colorMigration'] == null || this.scenesMap['AmazingFeature_colorMigration'].visible == false)
                return;
            if (this.lutTexture != null && this.targetUseCache == 1 && this.sourceUseCache == 1)
            {
                algorithm.setAlgorithmEnable(graphName, "general_lens_0", false);
                algorithm.setAlgorithmEnable(graphName, "general_lens_1", false);
                algorithm.setAlgorithmEnable(graphName, "general_lens_2", false);

                algorithm.setAlgorithmParamInt(graphName, "general_lens_2", "colormatch_is_update", 0);
                return;
            }
            else
            {
                algorithm.setAlgorithmEnable(graphName, "general_lens_0", true);
                algorithm.setAlgorithmEnable(graphName, "general_lens_1", true);
                algorithm.setAlgorithmEnable(graphName, "general_lens_2", true);
                this.lutTexture = null;
    
                algorithm.setAlgorithmParamStr(graphName, "general_lens_0", "colormatch_cache_path", this.targetPath);
                algorithm.setAlgorithmParamStr(graphName, "general_lens_1", "colormatch_cache_path", this.sourcePath);

                algorithm.setAlgorithmParamInt(graphName, "general_lens_0", "colormatch_use_cache", this.targetUseCache);
                algorithm.setAlgorithmParamInt(graphName, "general_lens_1", "colormatch_use_cache", 1);

                algorithm.setAlgorithmParamInt(graphName, "general_lens_2", "colormatch_is_update", 1);
            }
        } 
        if (graphName == 'smartColorAdjustment_VVdcRcCbhcmbVcfcCb06cIbccKb5c'){
            if (this.scenesMap['AmazingFeature_smartColorAdjustment'] == null || this.scenesMap['AmazingFeature_smartColorAdjustment'].visible == false)
                return;
            if (this.scenesMap['AmazingFeature_smartColorAdjustment'].getSceneOETF() == 1 || this.scenesMap['AmazingFeature_smartColorAdjustment'].getSceneOETF() == 2)
            {
                Amaz.AmazingManager.getSingleton("Algorithm").setAlgorithmParamStr(graphName, "vhdr_0", "execParam", "enhanceStrength=1.0&luminanceTarget=200");
            }
            else
            {
                Amaz.AmazingManager.getSingleton("Algorithm").setAlgorithmParamStr(graphName, "vhdr_0", "execParam", "enhanceStrength=1.0&luminanceTarget=250");
            }
        }
        if (graphName == 'colorCorrection_qbbkcvMbOblbVdqcfb8ibKdaeDd5'){
            if (this.scenesMap['AmazingFeature_colorCorrection'] == null || this.scenesMap['AmazingFeature_colorCorrection'].visible == false)
                return;
            if (this.isFirstFrame)
            {
                // console.warn("colorCorrection", "this.isFirstFrame = true.");
                algorithm.setAlgorithmEnable(graphName, "lens_ii_auto_color", true);
                this.isFirstFrame = false;
            }
            else
            {
                // console.warn("colorCorrection", "this.isFirstFrame = false.");
                algorithm.setAlgorithmEnable(graphName, "lens_ii_auto_color", false);
            }
        }
    }

    onAfterAlgorithmUpdate(graphName)
    {
        if (graphName == 'colormigration_hauisdhfiuwfhaui') {
            if (this.scenesMap['AmazingFeature_colorMigration'] == null || this.scenesMap['AmazingFeature_colorMigration'].visible == false)
                return;
            this.targetUseCache = 1;
            this.sourceUseCache = 1;
     
            // get texture from algorithm result.
            var lutTexture = this.getAlgorithmLutTexture(graphName);
            if (lutTexture == null)
                return;
        
            // create lut texture if needed.
            if (this.lutTexture == null)
            {
                this.lutTexture = this.createRenderTexture(lutTexture.width, lutTexture.height);
                this.lutTexture.isColorTexture = true;

                // blit texture
                var commandBuffer = new Amaz.CommandBuffer;
                commandBuffer.blit(lutTexture, this.lutTexture);
                this.scenesMap['AmazingFeature_colorMigration'].commitCommandBuffer(commandBuffer);
            }

            // set lut texture into render.
            this.colorMigrationMaterial.setTex("u_lut", this.lutTexture);
        }
    }

    getAlgorithmLutTexture(graphName)
    {
        var algorithm = Amaz.AmazingManager.getSingleton("Algorithm")
        var algorithmResult = algorithm.getAEAlgorithmResult();
        if (algorithmResult == null)
        {
            // console.warn("ColorMigration", "algorithm result is null.");
            return;
        }
        var algorithmInfo = algorithmResult.getAlgorithmInfo(graphName, "general_lens_2", "", 0);
        if (algorithmInfo == null)
        {
            // console.warn("ColorMigration", "algorithm information is null.");
            return;
        }
        var lutTexture = algorithmInfo.texture;
        if (lutTexture == null)
        {
            // console.warn("ColorMigration", "algorithm lut texture is null.");
            return;
        }
        var lutTexId = lutTexture.texId; 
        if (lutTexId <= 0)
        {
            // console.warn("ColorMigration", "algorithm lut texture id is invalid.");
            return;
        }
        return lutTexture;
    }
   
    createRenderTexture(width, height)
    {
        var texture = new Amaz.RenderTexture;
        texture.name = "lut_texture";
        texture.builtinType = Amaz.BuiltInTextureType.NORMAL;
        texture.internalFormat = Amaz.InternalFormat.RGBA8;
        texture.dataType = Amaz.DataType.U8norm;
        texture.width = width;
        texture.height = height;
        return texture;
    }
}

exports.Feature = Feature;