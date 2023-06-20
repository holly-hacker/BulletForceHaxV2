// handle CrazyGames integration
window.Crazygames = {
    gameplayStart: function () { console.log("called gameplayStart", arguments) },
    gameplayStop: function () { console.log("called gameplayStop", arguments) },
    screenshotReceived: function () { console.log("called screenshotReceived", arguments) },
    happytime: function () { console.log("called happytime", arguments) },
    init: function () { console.log("called init", arguments); },
    requestAd: function () { console.log("called requestAd", arguments) },
    requestBanners: function () { console.log("called requestBanners", arguments) },
    requestInviteUrl: function () { console.log("called requestInviteUrl", arguments) },
};

function ptr_to_str(ptr) {
    return window.gameModule.ccall("$fake_ccall$", "string", ["number"], [ptr]);
}

// NOTE: you can only allocate a single string per hook
function str_to_ptr(str) {
    return window.gameModule.ccall("$fake_ccall$", "number", ["string"], [str]);
}

function processWasmImports(imports) {
    console.log("Processing wasm imports", imports);


    for (const fn in imports.a) {
        const name = imports.a[fn].name;

        if (name == '_JS_Eval_EvalJS') {
            imports.a[fn] = (arg) => { console.log("EVAL JS", ptr_to_str(arg)) };
        }

        if (name == '_JS_Log_Dump') {
            imports.a[fn] = (arg) => { console.log("_JS_Log_Dump", ptr_to_str(arg)) };
        }

        if (name == '_JS_Log_StackTrace') {
            imports.a[fn] = function (arg) { console.log("_JS_Log_StackTrace", ptr_to_str(arg)/*, window.gameModule.stackTrace() */); };
        }

        if (bfh_config.hax) {
            if (name == '_SocketCreate') {
                console.log('hooking _SocketCreate');
                let orig_socketCreate = imports.a[fn];
                imports.a[fn] = function (url, protocols) {
                    // NOTE: I can pass the protocol to the proxy as well, that may be useful in the future
                    url = str_to_ptr(`ws://${window.location.host}/socket?${ptr_to_str(url)}`);
                    let ret = orig_socketCreate(url, protocols);
                    console.log("_SocketCreate", ptr_to_str(url), ptr_to_str(protocols), ret);
                    return ret;
                }
            }
        }

        if (bfh_config.hax && bfh_config.hax_http) {
            if (name == '_JS_WebRequest_Create') {
                console.log('hooking _JS_WebRequest_Create');
                let orig_webRequestCreate = imports.a[fn];
                imports.a[fn] = function (url, method) {
                    url = str_to_ptr(`${window.location.origin}/request?${ptr_to_str(url)}`);
                    let ret = orig_webRequestCreate(url, method);
                    console.log("_JS_WebRequest_Create", ptr_to_str(url), ptr_to_str(method), ret);
                    return ret;
                }
            }
        }
    }
}

let orig_instantiate = WebAssembly.instantiate;
WebAssembly.instantiate = function (source, importObject) {
    console.log("instantiating wasm object");
    processWasmImports(importObject);
    return orig_instantiate(source, importObject);
}

let orig_instantiateStreaming = WebAssembly.instantiateStreaming;
WebAssembly.instantiateStreaming = function (source, importObject) {
    console.log("instantiateStreaming wasm object");
    processWasmImports(importObject);
    return orig_instantiateStreaming(source, importObject);
}

fetch('/config.json')
    .then((response) => response.json())
    .then(bfh_config => {
        // document.querySelector('#loader-js').addEventListener('load', function () {
        load_game(bfh_config);
        // });
    })

function load_game(bfh_config) {
    console.log("BFH config", bfh_config);
    window.bfh_config = bfh_config;

    // instantiation logic taken from:
    // https://gist.github.com/davidjmcclelland/09b25b6baf6e0b09070c307d765b9f91

    let config = {
        dataUrl: '/game_assets/data.data',
        frameworkUrl: '/game_assets/framework.js',
        codeUrl: '/game_assets/code.wasm',
        showBanner: function () { console.log("unity banner", arguments); },

        // identity function to convert between numbers and strings
        "_$fake_ccall$": x => x,

        preRun: [function (mod) {
            window.gameModule = mod;
            console.log("preRun", arguments);
        }],
        postRun: [function (mod) { console.log("postRun", arguments); }],
    };

    createUnityInstance(document.querySelector("#unity-canvas"), config, (progress) => {
        // console.log("unity progress", progress);
    }).then((unityInstance) => {
        console.log("unity loaded", unityInstance);
    }).catch((message) => {
        alert(message);
    });

}
