const Algorithm = require("./quhui.js")
const Amaz = effect.Amaz;

class MainSystem {
  constructor() {
    this.name = 'MainSystem';
    this.params = new Object();
  }

  doLoadModel(model) { 
    // switch (model.name) {
    //   case this.inference_engine.engineD.modelName:
    //     this.inference_engine.engineD.loadModel(model);
			
    // }
  }

  // param0 alg : type JsWrapScriptAlgorithm
  doInit(alg) {
    this.MainSystemAlg = alg;
    alg.addInputType(0, Amaz.AlgorithmResultType.BLIT_IMAGE_BUFFER);
    // add a input type to this algo
    alg.addInputType(1, Amaz.AlgorithmResultType.SKIN_SEG);
    alg.addOutputType(0, Amaz.AlgorithmResultType.SCRIPT);

    this.alg = new Algorithm.quhui(this.params, this.systemParams, alg);
  }

  doDestroy() {
		
  }

  doApply(dirtyParams) {
    let filterKeySet = new Set(["base_skip_frames", "sticker_root", "script_path"]);

    let keySet = dirtyParams.getVectorKeys();

    for (let i = 0; i < keySet.size(); ++i) {
      let theKey = keySet.get(i);
      let theValue = dirtyParams.get(theKey);
      console.log("[DEBUG] applying params: " + theKey + " " + theValue);

      if (this.params[theKey] != theValue) {
        if (!filterKeySet.has(theKey)) {

          let func = this.alg.paramChangeCallbackMap.get(theKey);
          if(func != null) func(theValue)

          // this.alg.paramChangeCallbackMap.get(theKey)(theValue);
        }
      }
    }
  }

  // nodeContext : JSWrapNodeContext
  doExecute(nodeContext) { 
    let input = nodeContext.getInputResult();
    let output = nodeContext.getOutputMap();
    
    this.alg.execute(input, output);
  }

}
exports.MainSystem = MainSystem;