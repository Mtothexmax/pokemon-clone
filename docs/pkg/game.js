export function main() {
    wasm.main();
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Q_b78acf5ae4e73865: function(arg0) {
            const ret = getObject(arg0).Q;
            return addHeapObject(ret);
        },
        __wbg___wbindgen_boolean_get_5b446f51afd21013: function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_4687d8d8c2017d52: function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_function_1f9d30630b8b1d3d: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_e343b7d08827ba72: function(arg0) {
            const ret = getObject(arg0) === null;
            return ret;
        },
        __wbg___wbindgen_is_object_3c45d4f2dde4e749: function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_undefined_8865fb403f8fe9d8: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_number_get_2e0e7dee9f701a71: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_0380ccaa2f57f0d9: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_41e9ee4f547fc59a: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_dcc1a90847f04c41: function(arg0) {
            getObject(arg0)._wbg_cb_unref();
        },
        __wbg_addColorStop_2fe19c9df420d0a6: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).addColorStop(arg1, getStringFromWasm0(arg2, arg3));
        }, arguments); },
        __wbg_addEventListener_4d0db17c671ea324: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments); },
        __wbg_addEventListener_8d3235978b9c69a8: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), getObject(arg4));
        }, arguments); },
        __wbg_appendChild_fb8c52e7dd8484ea: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).appendChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_arcTo_a1ca709a1aae1a9b: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).arcTo(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_arc_03f7717b2e38682a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).arc(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_arrayBuffer_f07be74819944ea2: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).arrayBuffer();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_axes_1544d2030f316eff: function(arg0) {
            const ret = getObject(arg0).axes;
            return addHeapObject(ret);
        },
        __wbg_beginComputePass_7d30b3357d0cc1a3: function(arg0) {
            const ret = getObject(arg0).beginComputePass();
            return addHeapObject(ret);
        },
        __wbg_beginComputePass_8d343e0e3657e88c: function(arg0, arg1) {
            const ret = getObject(arg0).beginComputePass(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_beginPath_8598d895c13f1c86: function(arg0) {
            getObject(arg0).beginPath();
        },
        __wbg_beginPath_da1544c15d50142a: function(arg0) {
            getObject(arg0).beginPath();
        },
        __wbg_beginRenderPass_46bc75d13eaf7381: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).beginRenderPass(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_body_e549239eaff082e1: function(arg0) {
            const ret = getObject(arg0).body;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_buffer_56ec2905a66f58b9: function(arg0) {
            const ret = getObject(arg0).buffer;
            return addHeapObject(ret);
        },
        __wbg_buffered_6edc081be527f6fa: function(arg0) {
            const ret = getObject(arg0).buffered;
            return addHeapObject(ret);
        },
        __wbg_button_1aa380c7ca420cd1: function(arg0) {
            const ret = getObject(arg0).button;
            return ret;
        },
        __wbg_buttons_d32d3eb4f6cb5c35: function(arg0) {
            const ret = getObject(arg0).buttons;
            return addHeapObject(ret);
        },
        __wbg_call_187d372bd5fdd4aa: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_cancelScheduledValues_9b9e6893d788ea1c: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).cancelScheduledValues(arg1);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_className_0a31fdff102ecef8: function(arg0, arg1) {
            const ret = getObject(arg1).className;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_clearRect_0017d94d9f8e481a: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).clearRect(arg1, arg2, arg3, arg4);
        },
        __wbg_clearRect_a5e36505218c247a: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).clearRect(arg1, arg2, arg3, arg4);
        },
        __wbg_click_d990e70c576b7c54: function(arg0) {
            getObject(arg0).click();
        },
        __wbg_clientHeight_33dd2a2c8630a1f1: function(arg0) {
            const ret = getObject(arg0).clientHeight;
            return ret;
        },
        __wbg_clientWidth_08d5512595aacb73: function(arg0) {
            const ret = getObject(arg0).clientWidth;
            return ret;
        },
        __wbg_clientX_667e5633f888238d: function(arg0) {
            const ret = getObject(arg0).clientX;
            return ret;
        },
        __wbg_clientY_2cdc00632f185e67: function(arg0) {
            const ret = getObject(arg0).clientY;
            return ret;
        },
        __wbg_clip_275e70d2dc182fec: function(arg0) {
            getObject(arg0).clip();
        },
        __wbg_clipboard_9bf6a11ffd0b00a8: function(arg0) {
            const ret = getObject(arg0).clipboard;
            return addHeapObject(ret);
        },
        __wbg_closePath_355eb6da179dd9c2: function(arg0) {
            getObject(arg0).closePath();
        },
        __wbg_close_355afba6ea5ad1ae: function(arg0) {
            getObject(arg0).close();
        },
        __wbg_closest_6abd1bf5d7559cda: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).closest(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_code_422d539f5d7bacbb: function(arg0, arg1) {
            const ret = getObject(arg1).code;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_code_968c4f57c5d62cb4: function(arg0) {
            const ret = getObject(arg0).code;
            return ret;
        },
        __wbg_complete_e501182ac1c4e1e3: function(arg0) {
            const ret = getObject(arg0).complete;
            return ret;
        },
        __wbg_configure_a4e26031ba2da6a2: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).configure(getObject(arg1));
        }, arguments); },
        __wbg_connect_30bfff6aec59fc83: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).connect(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_contains_4222f17acc5763a3: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).contains(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_convertToBlob_ce4dcf84d91118e0: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).convertToBlob();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_copyBufferToBuffer_131a8b2ab6686279: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).copyBufferToBuffer(getObject(arg1), arg2 >>> 0, getObject(arg3), arg4 >>> 0);
        }, arguments); },
        __wbg_copyBufferToBuffer_6625868467f21a1d: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).copyBufferToBuffer(getObject(arg1), arg2 >>> 0, getObject(arg3), arg4 >>> 0, arg5 >>> 0);
        }, arguments); },
        __wbg_copyExternalImageToTexture_10bb705141e3293b: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).copyExternalImageToTexture(getObject(arg1), getObject(arg2), getArrayJsValueViewFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_copyTextureToBuffer_d7dbab5b34fdfd80: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).copyTextureToBuffer(getObject(arg1), getObject(arg2), getArrayJsValueViewFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_copyTextureToTexture_3f53edb84c2b8992: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).copyTextureToTexture(getObject(arg1), getObject(arg2), getArrayJsValueViewFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_copyToChannel_459cca8c62b0e606: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).copyToChannel(getArrayF32FromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_createBindGroupLayout_dd6a685e68699be9: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).createBindGroupLayout(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createBindGroup_297f726263493b7a: function(arg0, arg1) {
            const ret = getObject(arg0).createBindGroup(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createBiquadFilter_b67f3bb6101472ac: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).createBiquadFilter();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createBufferSource_2cbe6214f761dfcb: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).createBufferSource();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createBuffer_41ea68f490cd6844: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).createBuffer(arg1 >>> 0, arg2 >>> 0, arg3);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createBuffer_7d58b37b7042f31f: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).createBuffer(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createCommandEncoder_e9f5d561e0436021: function(arg0) {
            const ret = getObject(arg0).createCommandEncoder();
            return addHeapObject(ret);
        },
        __wbg_createComputePipeline_1efe4869c7158b32: function(arg0, arg1) {
            const ret = getObject(arg0).createComputePipeline(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createConvolver_0f6ccaf00992f8ae: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).createConvolver();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createDelay_7b4d817a72ba1efe: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).createDelay(arg1);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createElement_74049073a11f9c31: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createGain_b6551e1ad1609779: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).createGain();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createImageBitmap_1a53b9fe15f242e6: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).createImageBitmap(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createLinearGradient_d67a7d18d11c0b62: function(arg0, arg1, arg2, arg3, arg4) {
            const ret = getObject(arg0).createLinearGradient(arg1, arg2, arg3, arg4);
            return addHeapObject(ret);
        },
        __wbg_createMediaElementSource_f7fc575d58a891fb: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).createMediaElementSource(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createObjectStore_84e0f5101158c82d: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).createObjectStore(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createObjectURL_2f3b38279fe61801: function() { return handleError(function (arg0, arg1) {
            const ret = URL.createObjectURL(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_createPanner_3dedf22bb77a061c: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).createPanner();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createPipelineLayout_39a9a7f694e28389: function(arg0, arg1) {
            const ret = getObject(arg0).createPipelineLayout(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createQuerySet_dc3d63f5349d8c51: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).createQuerySet(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createRadialGradient_e54676bfd0a6ae40: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            const ret = getObject(arg0).createRadialGradient(arg1, arg2, arg3, arg4, arg5, arg6);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createRenderPipeline_e7c57f75917917e4: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).createRenderPipeline(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createSampler_1271407d57de5444: function(arg0, arg1) {
            const ret = getObject(arg0).createSampler(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createShaderModule_274373c4cacce3a0: function(arg0, arg1) {
            const ret = getObject(arg0).createShaderModule(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createStereoPanner_862be1e9bbb5f4eb: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).createStereoPanner();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createTexture_272b1f1151792884: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).createTexture(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createView_56a084407ee096a1: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).createView(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createView_7520397d720e9d7d: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).createView();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createWaveShaper_a4048d67dcfd9e13: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).createWaveShaper();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_currentTime_baa21b24faf1f050: function(arg0) {
            const ret = getObject(arg0).currentTime;
            return ret;
        },
        __wbg_currentTime_eb1592316a1d4450: function(arg0) {
            const ret = getObject(arg0).currentTime;
            return ret;
        },
        __wbg_data_9ce11d34b90054d4: function(arg0, arg1) {
            const ret = getObject(arg1).data;
            const ptr1 = passArray8ToWasm0(ret, wasm.__wbindgen_export);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_decodeAudioData_ebde00be7f48c9ec: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).decodeAudioData(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_delayTime_637780cd4a3c6ada: function(arg0) {
            const ret = getObject(arg0).delayTime;
            return addHeapObject(ret);
        },
        __wbg_delete_efcc1a7f91e815b8: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).delete(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_deltaY_7abc8fc9878d0002: function(arg0) {
            const ret = getObject(arg0).deltaY;
            return ret;
        },
        __wbg_destination_05471d87031cba86: function(arg0) {
            const ret = getObject(arg0).destination;
            return addHeapObject(ret);
        },
        __wbg_destroy_ddbf2970cd27b6ec: function(arg0) {
            getObject(arg0).destroy();
        },
        __wbg_destroy_f83fd85ed8ee84cb: function(arg0) {
            getObject(arg0).destroy();
        },
        __wbg_detune_460c48c845d51dd0: function(arg0) {
            const ret = getObject(arg0).detune;
            return addHeapObject(ret);
        },
        __wbg_devicePixelRatio_7d39e9af5448d3d4: function(arg0) {
            const ret = getObject(arg0).devicePixelRatio;
            return ret;
        },
        __wbg_disconnect_513def74b1e0d5cc: function() { return handleError(function (arg0) {
            getObject(arg0).disconnect();
        }, arguments); },
        __wbg_dispatchEvent_57e1a8af186ff471: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).dispatchEvent(getObject(arg1));
            return ret;
        }, arguments); },
        __wbg_dispatchWorkgroups_e921b9b1171d85bf: function(arg0, arg1) {
            getObject(arg0).dispatchWorkgroups(arg1 >>> 0);
        },
        __wbg_dispatchWorkgroups_edc36bb2fb80edfc: function(arg0, arg1, arg2) {
            getObject(arg0).dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_document_9854e03c05fc8834: function(arg0) {
            const ret = getObject(arg0).document;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_drawImage_61d43245950ef561: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).drawImage(getObject(arg1), arg2, arg3);
        }, arguments); },
        __wbg_drawImage_6f841fbe7ccefe78: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).drawImage(getObject(arg1), arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_drawImage_76047a46c1731363: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).drawImage(getObject(arg1), arg2, arg3);
        }, arguments); },
        __wbg_drawImage_c0004b34389582f6: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).drawImage(getObject(arg1), arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_drawIndexedIndirect_9ec1ae37cdda8f55: function(arg0, arg1, arg2) {
            getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2 >>> 0);
        },
        __wbg_drawIndexed_2830e7be41ed5761: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
        },
        __wbg_drawIndexed_b414a9de781296f1: function(arg0, arg1) {
            getObject(arg0).drawIndexed(arg1 >>> 0);
        },
        __wbg_draw_2776aa4d77902656: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_draw_889697bdf4cad1b7: function(arg0, arg1, arg2) {
            getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_draw_b3ff1fa9ff891153: function(arg0, arg1) {
            getObject(arg0).draw(arg1 >>> 0);
        },
        __wbg_duration_112842087d5c0981: function(arg0) {
            const ret = getObject(arg0).duration;
            return ret;
        },
        __wbg_duration_f2b08ee224fbf12e: function(arg0) {
            const ret = getObject(arg0).duration;
            return ret;
        },
        __wbg_end_1ad40f5d5c98be6d: function(arg0) {
            getObject(arg0).end();
        },
        __wbg_end_7f76e01bb29a0e95: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).end(arg1 >>> 0);
            return ret;
        }, arguments); },
        __wbg_end_a97f51ac1f092bfe: function(arg0) {
            getObject(arg0).end();
        },
        __wbg_error_2404b46258983c95: function(arg0) {
            const ret = getObject(arg0).error;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_error_6a23b57453026a47: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).error;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_error_757e9472f8410341: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_error_c9cf3fc2064683a9: function(arg0) {
            console.error(getObject(arg0));
        },
        __wbg_error_fb1c947d6e1e6ae6: function(arg0) {
            const ret = getObject(arg0).error;
            return addHeapObject(ret);
        },
        __wbg_features_2c4eceb4ede31583: function(arg0) {
            const ret = getObject(arg0).features;
            return addHeapObject(ret);
        },
        __wbg_fetch_4178962c570137ca: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).fetch(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        },
        __wbg_fetch_b322877f1ed6457d: function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).fetch(getStringFromWasm0(arg1, arg2), getObject(arg3));
            return addHeapObject(ret);
        },
        __wbg_files_cb1ce1b82f16c484: function(arg0) {
            const ret = getObject(arg0).files;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_fillRect_0ef59adb9acb7d06: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).fillRect(arg1, arg2, arg3, arg4);
        },
        __wbg_fillRect_23cfe0a2e8af97a0: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).fillRect(arg1, arg2, arg3, arg4);
        },
        __wbg_fillText_1bcec8b81ad73bd0: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).fillText(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_fillText_8e25ab4d191bc3ad: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).fillText(getStringFromWasm0(arg1, arg2), arg3, arg4, arg5);
        }, arguments); },
        __wbg_fill_7bfe4a90ca0e1b05: function(arg0) {
            getObject(arg0).fill();
        },
        __wbg_fill_fc5e02a06cc26e92: function(arg0) {
            getObject(arg0).fill();
        },
        __wbg_finish_489db1750712613c: function(arg0) {
            const ret = getObject(arg0).finish();
            return addHeapObject(ret);
        },
        __wbg_focus_f740d61348f422e7: function() { return handleError(function (arg0) {
            getObject(arg0).focus();
        }, arguments); },
        __wbg_frequency_ce702a1bc542b592: function(arg0) {
            const ret = getObject(arg0).frequency;
            return addHeapObject(ret);
        },
        __wbg_from_296ca31f8d0f1c52: function(arg0) {
            const ret = Array.from(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_gain_29777386775059c0: function(arg0) {
            const ret = getObject(arg0).gain;
            return addHeapObject(ret);
        },
        __wbg_getAllKeys_2540c47f21cb3f26: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).getAllKeys();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_getAll_ed90603b488e51b6: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).getAll();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_getAttribute_061ad00c16e2f622: function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getAttribute(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getBindGroupLayout_208b6005a9dd0c87: function(arg0, arg1) {
            const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_getBindGroupLayout_a73b8b26696c272a: function(arg0, arg1) {
            const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_getBoundingClientRect_57152b1a20f3de34: function(arg0) {
            const ret = getObject(arg0).getBoundingClientRect();
            return addHeapObject(ret);
        },
        __wbg_getChannelData_56e164370efbb263: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg1).getChannelData(arg2 >>> 0);
            const ptr1 = passArrayF32ToWasm0(ret, wasm.__wbindgen_export);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getCompilationInfo_c23e65cdc88c7f8a: function(arg0) {
            const ret = getObject(arg0).getCompilationInfo();
            return addHeapObject(ret);
        },
        __wbg_getContext_635e36719cad2623: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_getContext_e567868594d2e0c6: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_getCurrentTexture_f9b3ddbfb8575837: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).getCurrentTexture();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_getDate_1181c785ea4fdd21: function(arg0) {
            const ret = getObject(arg0).getDate();
            return ret;
        },
        __wbg_getElementById_cc94972b404e4eaa: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_getFullYear_32050b2c56c56cd8: function(arg0) {
            const ret = getObject(arg0).getFullYear();
            return ret;
        },
        __wbg_getGamepads_912bbd8a90a857f7: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).getGamepads();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_getHours_f57dcda3efee150c: function(arg0) {
            const ret = getObject(arg0).getHours();
            return ret;
        },
        __wbg_getImageData_3c5faf3b3dc6124f: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = getObject(arg0).getImageData(arg1, arg2, arg3, arg4);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_getImageData_e3d068a7241cd3d4: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = getObject(arg0).getImageData(arg1, arg2, arg3, arg4);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_getItem_4c80b7663197c7ff: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getMappedRange_7f87f943c01c0b0f: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).getMappedRange(arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_getMappedRange_acd1b5b90bf13479: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).getMappedRange();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_getMinutes_951e7f98fc4fdb30: function(arg0) {
            const ret = getObject(arg0).getMinutes();
            return ret;
        },
        __wbg_getMonth_aae5f530fd173536: function(arg0) {
            const ret = getObject(arg0).getMonth();
            return ret;
        },
        __wbg_getPreferredCanvasFormat_4d28b652aa67cc74: function(arg0) {
            const ret = getObject(arg0).getPreferredCanvasFormat();
            return (__wbindgen_enum_GpuTextureFormat.indexOf(ret) + 1 || 102) - 1;
        },
        __wbg_getSeconds_8ffcca0e220547af: function(arg0) {
            const ret = getObject(arg0).getSeconds();
            return ret;
        },
        __wbg_get_31af05bd4842a84f: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_get_5dde8a29f073d908: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_get_6c896e0571ddae51: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        },
        __wbg_get_index_c78b0af767cc90df: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return ret;
        },
        __wbg_get_unchecked_288889d017702237: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        },
        __wbg_gpu_d217bb3813d12cf1: function(arg0) {
            const ret = getObject(arg0).gpu;
            return addHeapObject(ret);
        },
        __wbg_hasAttribute_b009da6c546736e9: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).hasAttribute(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_hasFocus_a0ae804216f45b06: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).hasFocus();
            return ret;
        }, arguments); },
        __wbg_has_43e0b32ce4e12555: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).has(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_height_9dca4d43c3e4b84b: function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        },
        __wbg_height_bd5d010e070a30f4: function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        },
        __wbg_height_fb13ed9fe991b5f4: function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        },
        __wbg_height_fc97e1a0c2e7331f: function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        },
        __wbg_hidden_2ba371ecafe013fa: function(arg0) {
            const ret = getObject(arg0).hidden;
            return ret;
        },
        __wbg_indexedDB_53d75531b7daa0cd: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).indexedDB;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_instanceof_Blob_7c6191a4cd5834d3: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Blob;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_CanvasRenderingContext2d_769208c72dcbf5e6: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof CanvasRenderingContext2D;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Element_818e11074cdb63b5: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Element;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Error_80a725f81f2e102d: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Error;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GamepadButton_194af99e75be79f5: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof GamepadButton;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Gamepad_379e1510b278c518: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Gamepad;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuCompilationMessage_4b18a454edec05ec: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof GPUCompilationMessage;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuError_efa82ee186207332: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof GPUError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlAnchorElement_e2e601da36602959: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLAnchorElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlCanvasElement_0a30c11fbbf41841: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlElement_32fb153a5a0e2349: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlInputElement_5c33d1de59c09c49: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLInputElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_IdbDatabase_ee9e122a32a24d91: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof IDBDatabase;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_ImageBitmap_74bea45ee48d3d7a: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ImageBitmap;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_82d71df4eddf88bc: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_key_1193871533b99ae5: function(arg0, arg1) {
            const ret = getObject(arg1).key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_key_665a26f1e015e4db: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg1).key(arg2 >>> 0);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_left_e0a244490fe2f293: function(arg0) {
            const ret = getObject(arg0).left;
            return ret;
        },
        __wbg_length_373463a7969053f2: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_4214f4ab2a24bbfc: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).length;
            return ret;
        }, arguments); },
        __wbg_length_7f3c00c40364105e: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_bed32df35e869dbb: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_d4bdea10311bd9cf: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_lineNum_0a6ea0f09515536b: function(arg0) {
            const ret = getObject(arg0).lineNum;
            return ret;
        },
        __wbg_linePos_d0a71dbe45cdecff: function(arg0) {
            const ret = getObject(arg0).linePos;
            return ret;
        },
        __wbg_lineTo_63fac7d60279d95e: function(arg0, arg1, arg2) {
            getObject(arg0).lineTo(arg1, arg2);
        },
        __wbg_linearRampToValueAtTime_f87a0c73867277b7: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).linearRampToValueAtTime(arg1, arg2);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_listener_82c655ccaabeebde: function(arg0) {
            const ret = getObject(arg0).listener;
            return addHeapObject(ret);
        },
        __wbg_load_0177183b4a32f746: function(arg0) {
            getObject(arg0).load();
        },
        __wbg_localStorage_567c24950edbc178: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).localStorage;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_location_9c46e8ada95d3174: function(arg0) {
            const ret = getObject(arg0).location;
            return addHeapObject(ret);
        },
        __wbg_log_17c30ef363c61cf4: function(arg0) {
            console.log(getObject(arg0));
        },
        __wbg_loop_889b52c51cd79d21: function(arg0) {
            const ret = getObject(arg0).loop;
            return ret;
        },
        __wbg_mapAsync_04287bb229a8a839: function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).mapAsync(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_mapAsync_baa5e65683873fff: function(arg0, arg1) {
            const ret = getObject(arg0).mapAsync(arg1 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_matchMedia_8a4857f947f11f82: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_matches_778d0873b064b1bf: function(arg0) {
            const ret = getObject(arg0).matches;
            return ret;
        },
        __wbg_maxTouchPoints_2d6c5004201091b8: function(arg0) {
            const ret = getObject(arg0).maxTouchPoints;
            return ret;
        },
        __wbg_measureText_1035b288be51876c: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).measureText(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_message_5f8387f0c32b90a7: function(arg0) {
            const ret = getObject(arg0).message;
            return addHeapObject(ret);
        },
        __wbg_message_83dde86f5fa85867: function(arg0, arg1) {
            const ret = getObject(arg1).message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_message_d988ea596c5f5c9a: function(arg0, arg1) {
            const ret = getObject(arg1).message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_message_fb5bb3f5aeb873e4: function(arg0, arg1) {
            const ret = getObject(arg1).message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_messages_28b1d70e8e2fba0e: function(arg0) {
            const ret = getObject(arg0).messages;
            return addHeapObject(ret);
        },
        __wbg_moveTo_037e3deefec91ae1: function(arg0, arg1, arg2) {
            getObject(arg0).moveTo(arg1, arg2);
        },
        __wbg_movementX_b1d3c3973679d356: function(arg0) {
            const ret = getObject(arg0).movementX;
            return ret;
        },
        __wbg_movementY_ec06af437dd2f47b: function(arg0) {
            const ret = getObject(arg0).movementY;
            return ret;
        },
        __wbg_naturalHeight_df41ba6a4a6b0d33: function(arg0) {
            const ret = getObject(arg0).naturalHeight;
            return ret;
        },
        __wbg_naturalWidth_c4c567dc88a9ee9a: function(arg0) {
            const ret = getObject(arg0).naturalWidth;
            return ret;
        },
        __wbg_navigator_2156486643462a87: function(arg0) {
            const ret = getObject(arg0).navigator;
            return addHeapObject(ret);
        },
        __wbg_networkState_a70079971b8231c6: function(arg0) {
            const ret = getObject(arg0).networkState;
            return ret;
        },
        __wbg_new_0_72d020f0c63443d4: function() {
            const ret = new Date();
            return addHeapObject(ret);
        },
        __wbg_new_1dbf7428bba60a42: function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return addHeapObject(ret);
        },
        __wbg_new_5502aad30c185fc8: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return __wasm_bindgen_func_elem_3393(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return addHeapObject(ret);
            } finally {
                state0.a = 0;
            }
        },
        __wbg_new_617a8cdb8bb1130e: function() {
            const ret = new Object();
            return addHeapObject(ret);
        },
        __wbg_new_74cd12440f4b8e4e: function(arg0) {
            const ret = new Uint32Array(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_new_83f09a1ace2e582d: function() { return handleError(function () {
            const ret = new Image();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_a217df4351db4b9b: function() { return handleError(function (arg0, arg1) {
            const ret = new OffscreenCanvas(arg0 >>> 0, arg1 >>> 0);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_ae4555a03ebb1118: function() { return handleError(function () {
            const ret = new FileReader();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_afd97fa36645d359: function() { return handleError(function () {
            const ret = new lAudioContext();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_b5c815b5c2a0cb68: function() { return handleError(function (arg0, arg1) {
            const ret = new Event(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_ee2291f50781bf1d: function() {
            const ret = new Array();
            return addHeapObject(ret);
        },
        __wbg_new_from_slice_9a868026ffa4208a: function(arg0, arg1) {
            const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_new_with_src_262a7834694390e6: function() { return handleError(function (arg0, arg1) {
            const ret = new Audio(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_with_str_sequence_and_options_e6f31745d4b2e5e4: function() { return handleError(function (arg0, arg1) {
            const ret = new Blob(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_with_u8_array_sequence_and_options_fccc08fe93f4b8fd: function() { return handleError(function (arg0, arg1) {
            const ret = new Blob(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_with_u8_clamped_array_and_sh_af3f0dee40638e18: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = new ImageData(getClampedArrayU8FromWasm0(arg0, arg1), arg2 >>> 0, arg3 >>> 0);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_now_aa4ccb83129e9e55: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_now_d0b7f4bea9f38490: function(arg0) {
            const ret = getObject(arg0).now();
            return ret;
        },
        __wbg_numberOfChannels_14d34cb18d851ff8: function(arg0) {
            const ret = getObject(arg0).numberOfChannels;
            return ret;
        },
        __wbg_objectStoreNames_19a06aba400d0baf: function(arg0) {
            const ret = getObject(arg0).objectStoreNames;
            return addHeapObject(ret);
        },
        __wbg_objectStore_43178e7a3459e234: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).objectStore(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_of_d5dbecdc59e02e34: function(arg0) {
            const ret = Array.of(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_ok_125878a7db256ecb: function(arg0) {
            const ret = getObject(arg0).ok;
            return ret;
        },
        __wbg_onSubmittedWorkDone_d185d0b92c93b668: function(arg0) {
            const ret = getObject(arg0).onSubmittedWorkDone();
            return addHeapObject(ret);
        },
        __wbg_open_dbf85f9611885c1b: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).open(getStringFromWasm0(arg1, arg2), arg3 >>> 0);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_pan_6f3f912c554eb0ea: function(arg0) {
            const ret = getObject(arg0).pan;
            return addHeapObject(ret);
        },
        __wbg_parse_0fc53dead14b3b42: function() { return handleError(function (arg0, arg1) {
            const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_pause_dc581ba7b666b4a6: function() { return handleError(function (arg0) {
            getObject(arg0).pause();
        }, arguments); },
        __wbg_paused_1b8310f12cfad1c1: function(arg0) {
            const ret = getObject(arg0).paused;
            return ret;
        },
        __wbg_performance_d0d7b03fc649b694: function(arg0) {
            const ret = getObject(arg0).performance;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_play_86145c1e0cb32fb6: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).play();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_playbackRate_21b3870212519435: function(arg0) {
            const ret = getObject(arg0).playbackRate;
            return addHeapObject(ret);
        },
        __wbg_pointerId_1b91f6b6c8f92501: function(arg0) {
            const ret = getObject(arg0).pointerId;
            return ret;
        },
        __wbg_pointerLockElement_0697431b2d350d8c: function(arg0) {
            const ret = getObject(arg0).pointerLockElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_pointerType_2e8b8270da489520: function(arg0, arg1) {
            const ret = getObject(arg1).pointerType;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_popErrorScope_64398374fcf6a395: function(arg0) {
            const ret = getObject(arg0).popErrorScope();
            return addHeapObject(ret);
        },
        __wbg_positionX_f241ca61b68aa02a: function(arg0) {
            const ret = getObject(arg0).positionX;
            return addHeapObject(ret);
        },
        __wbg_positionY_1f15c2cdc8613caf: function(arg0) {
            const ret = getObject(arg0).positionY;
            return addHeapObject(ret);
        },
        __wbg_positionZ_44201e167021a66c: function(arg0) {
            const ret = getObject(arg0).positionZ;
            return addHeapObject(ret);
        },
        __wbg_pressed_e774166b0a6c08fc: function(arg0) {
            const ret = getObject(arg0).pressed;
            return ret;
        },
        __wbg_preventDefault_af59afb0f0a02e20: function(arg0) {
            getObject(arg0).preventDefault();
        },
        __wbg_prototypesetcall_bc27214492979395: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
        },
        __wbg_pushErrorScope_9489e9047c263086: function(arg0, arg1) {
            getObject(arg0).pushErrorScope(__wbindgen_enum_GpuErrorFilter[arg1]);
        },
        __wbg_push_2baf45db356cf468: function(arg0, arg1) {
            const ret = getObject(arg0).push(getObject(arg1));
            return ret;
        },
        __wbg_putImageData_61579e867341ed92: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).putImageData(getObject(arg1), arg2, arg3);
        }, arguments); },
        __wbg_putImageData_8d388310b86bafd8: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).putImageData(getObject(arg1), arg2, arg3);
        }, arguments); },
        __wbg_querySelector_49877e2a9e3f670b: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_querySelector_ecbc2550705b235f: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_queueMicrotask_9833f9a49df95a49: function(arg0) {
            const ret = getObject(arg0).queueMicrotask;
            return addHeapObject(ret);
        },
        __wbg_queueMicrotask_a72f977e97f23c5f: function(arg0) {
            queueMicrotask(getObject(arg0));
        },
        __wbg_queue_17ba8833afbd540b: function(arg0) {
            const ret = getObject(arg0).queue;
            return addHeapObject(ret);
        },
        __wbg_random_5a4cafd2f02395ff: function() {
            const ret = Math.random();
            return ret;
        },
        __wbg_readAsText_301ec1c6739475ea: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).readAsText(getObject(arg1));
        }, arguments); },
        __wbg_readyState_61ca59f494fa2626: function(arg0) {
            const ret = getObject(arg0).readyState;
            return ret;
        },
        __wbg_reload_0dd932ae5e23064a: function() { return handleError(function (arg0) {
            getObject(arg0).reload();
        }, arguments); },
        __wbg_removeAttribute_2f2700a6f933a6be: function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_removeItem_f7b602722ed566dd: function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).removeItem(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_remove_9a5288388316028c: function(arg0) {
            getObject(arg0).remove();
        },
        __wbg_repeat_8e70ccf8a9875d04: function(arg0) {
            const ret = getObject(arg0).repeat;
            return ret;
        },
        __wbg_requestAdapter_ed2ade66c5164166: function(arg0, arg1) {
            const ret = getObject(arg0).requestAdapter(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_requestAnimationFrame_7b526ab7aa550c74: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
            return ret;
        }, arguments); },
        __wbg_requestDevice_f099f03f8c1981ff: function(arg0, arg1) {
            const ret = getObject(arg0).requestDevice(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_resolveQuerySet_ba9834cc90f06618: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).resolveQuerySet(getObject(arg1), arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5 >>> 0);
        },
        __wbg_resolve_0076e10020304ede: function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_restore_c93ba7571816b182: function(arg0) {
            getObject(arg0).restore();
        },
        __wbg_result_66ae42c57a28369f: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).result;
            return addHeapObject(ret);
        }, arguments); },
        __wbg_result_ea60156c5cd4726f: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).result;
            return addHeapObject(ret);
        }, arguments); },
        __wbg_resume_262977d0e323b264: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).resume();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_revokeObjectURL_479ff72c2d109d80: function() { return handleError(function (arg0, arg1) {
            URL.revokeObjectURL(getStringFromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_roundRect_7a28b1c64cc87504: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).roundRect(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_sampleRate_e7f5086f72f12f2e: function(arg0) {
            const ret = getObject(arg0).sampleRate;
            return ret;
        },
        __wbg_sampleRate_f1a896dec84704f9: function(arg0) {
            const ret = getObject(arg0).sampleRate;
            return ret;
        },
        __wbg_save_f32554f1747071d1: function(arg0) {
            getObject(arg0).save();
        },
        __wbg_setAttribute_9e7d603908f63705: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setBindGroup_4af09961dd6fd6d4: function(arg0, arg1, arg2) {
            getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
        },
        __wbg_setBindGroup_d73d7d39d690dded: function(arg0, arg1, arg2) {
            getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
        },
        __wbg_setIndexBuffer_d9fac7388da9021a: function(arg0, arg1, arg2) {
            getObject(arg0).setIndexBuffer(getObject(arg1), __wbindgen_enum_GpuIndexFormat[arg2]);
        },
        __wbg_setItem_c5fb0966b484ecd3: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setOrientation_8606bfea1fd65552: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).setOrientation(arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_setPipeline_58877204a6051c42: function(arg0, arg1) {
            getObject(arg0).setPipeline(getObject(arg1));
        },
        __wbg_setPipeline_9870f87d9e0a49ab: function(arg0, arg1) {
            getObject(arg0).setPipeline(getObject(arg1));
        },
        __wbg_setPointerCapture_a53e23b7bccef290: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).setPointerCapture(arg1);
        }, arguments); },
        __wbg_setPosition_876d017d4b44c810: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).setPosition(arg1, arg2, arg3);
        },
        __wbg_setProperty_097bc3d55ce44513: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setScissorRect_78603d18fd6477a3: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_setTimeout_db7bbc18a17e152a: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
            return ret;
        }, arguments); },
        __wbg_setValueAtTime_2edc098944a654d2: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).setValueAtTime(arg1, arg2);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_setVertexBuffer_7e35a3ca5e6c4878: function(arg0, arg1, arg2) {
            getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2));
        },
        __wbg_setViewport_a10f0909ea1d3836: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_set_145a351398b48c65: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments); },
        __wbg_set_a_d2dae9a62946ba86: function(arg0, arg1) {
            getObject(arg0).a = arg1;
        },
        __wbg_set_accept_06d626c479e21e36: function(arg0, arg1, arg2) {
            getObject(arg0).accept = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_access_75fd5e68501830ce: function(arg0, arg1) {
            getObject(arg0).access = __wbindgen_enum_GpuStorageTextureAccess[arg1];
        },
        __wbg_set_address_mode_u_38bbe83e7cb70867: function(arg0, arg1) {
            getObject(arg0).addressModeU = __wbindgen_enum_GpuAddressMode[arg1];
        },
        __wbg_set_address_mode_v_e4dc81ae95d4e2d1: function(arg0, arg1) {
            getObject(arg0).addressModeV = __wbindgen_enum_GpuAddressMode[arg1];
        },
        __wbg_set_alpha_b14827cf8b4ac26a: function(arg0, arg1) {
            getObject(arg0).alpha = getObject(arg1);
        },
        __wbg_set_alpha_mode_16c3966c224ad4a9: function(arg0, arg1) {
            getObject(arg0).alphaMode = __wbindgen_enum_GpuCanvasAlphaMode[arg1];
        },
        __wbg_set_array_layer_count_89cae743069411e2: function(arg0, arg1) {
            getObject(arg0).arrayLayerCount = arg1 >>> 0;
        },
        __wbg_set_array_stride_c30294b65e05ecbf: function(arg0, arg1) {
            getObject(arg0).arrayStride = arg1 >>> 0;
        },
        __wbg_set_attributes_9f3751b5d4cb6fbf: function(arg0, arg1, arg2) {
            getObject(arg0).attributes = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_b_670dfc0631dcb9c1: function(arg0, arg1) {
            getObject(arg0).b = arg1;
        },
        __wbg_set_base_array_layer_ccd25abaf4bab74a: function(arg0, arg1) {
            getObject(arg0).baseArrayLayer = arg1 >>> 0;
        },
        __wbg_set_base_mip_level_b770d77868e48001: function(arg0, arg1) {
            getObject(arg0).baseMipLevel = arg1 >>> 0;
        },
        __wbg_set_beginning_of_pass_write_index_3c28a3725a66aaf4: function(arg0, arg1) {
            getObject(arg0).beginningOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_beginning_of_pass_write_index_a236195f84e9a61a: function(arg0, arg1) {
            getObject(arg0).beginningOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_bind_group_layouts_60a4869809302d79: function(arg0, arg1, arg2) {
            getObject(arg0).bindGroupLayouts = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_binding_1d51e1a2d1c0cdaf: function(arg0, arg1) {
            getObject(arg0).binding = arg1 >>> 0;
        },
        __wbg_set_binding_348f002e688081ac: function(arg0, arg1) {
            getObject(arg0).binding = arg1 >>> 0;
        },
        __wbg_set_blend_3f83603fc878c51d: function(arg0, arg1) {
            getObject(arg0).blend = getObject(arg1);
        },
        __wbg_set_body_1fb0f1008bfc7df6: function(arg0, arg1) {
            getObject(arg0).body = getObject(arg1);
        },
        __wbg_set_buffer_22331adbf81699d2: function(arg0, arg1) {
            getObject(arg0).buffer = getObject(arg1);
        },
        __wbg_set_buffer_677f77bef838f105: function(arg0, arg1) {
            getObject(arg0).buffer = getObject(arg1);
        },
        __wbg_set_buffer_6ce9ebe1f515ddb9: function(arg0, arg1) {
            getObject(arg0).buffer = getObject(arg1);
        },
        __wbg_set_buffer_7ab48e13895631dc: function(arg0, arg1) {
            getObject(arg0).buffer = getObject(arg1);
        },
        __wbg_set_buffer_b8d40a54af365605: function(arg0, arg1) {
            getObject(arg0).buffer = getObject(arg1);
        },
        __wbg_set_buffers_c1346a3c65c0829a: function(arg0, arg1, arg2) {
            getObject(arg0).buffers = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_bytes_per_row_95041361899e65c1: function(arg0, arg1) {
            getObject(arg0).bytesPerRow = arg1 >>> 0;
        },
        __wbg_set_bytes_per_row_9a64b83786d0ca8e: function(arg0, arg1) {
            getObject(arg0).bytesPerRow = arg1 >>> 0;
        },
        __wbg_set_className_541fce5cd31918aa: function(arg0, arg1, arg2) {
            getObject(arg0).className = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_clear_value_gpu_color_dict_c17321221601e286: function(arg0, arg1) {
            getObject(arg0).clearValue = getObject(arg1);
        },
        __wbg_set_code_4510e93549b3066e: function(arg0, arg1, arg2) {
            getObject(arg0).code = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_color_14d2fdcfc7117305: function(arg0, arg1) {
            getObject(arg0).color = getObject(arg1);
        },
        __wbg_set_color_attachments_367a5baf85bca057: function(arg0, arg1, arg2) {
            getObject(arg0).colorAttachments = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_compare_e4e04c3058d99f10: function(arg0, arg1) {
            getObject(arg0).compare = __wbindgen_enum_GpuCompareFunction[arg1];
        },
        __wbg_set_compute_61a7cc3c3e3fb7f6: function(arg0, arg1) {
            getObject(arg0).compute = getObject(arg1);
        },
        __wbg_set_constants_d754ac2933932062: function(arg0, arg1) {
            getObject(arg0).constants = getObject(arg1);
        },
        __wbg_set_count_1733859bce4413c6: function(arg0, arg1) {
            getObject(arg0).count = arg1 >>> 0;
        },
        __wbg_set_count_d8af002c4b4b808d: function(arg0, arg1) {
            getObject(arg0).count = arg1 >>> 0;
        },
        __wbg_set_cull_mode_7a9f5499205a7f66: function(arg0, arg1) {
            getObject(arg0).cullMode = __wbindgen_enum_GpuCullMode[arg1];
        },
        __wbg_set_curve_6a2e769a004d41ab: function(arg0, arg1, arg2) {
            getObject(arg0).curve = arg1 === 0 ? undefined : getArrayF32FromWasm0(arg1, arg2);
        },
        __wbg_set_depth_bias_1994cc72067a977b: function(arg0, arg1) {
            getObject(arg0).depthBias = arg1;
        },
        __wbg_set_depth_bias_slope_scale_7aa61e8c4189f10f: function(arg0, arg1) {
            getObject(arg0).depthBiasSlopeScale = arg1;
        },
        __wbg_set_depth_clear_value_5063e6b8ebc5e409: function(arg0, arg1) {
            getObject(arg0).depthClearValue = arg1;
        },
        __wbg_set_depth_compare_6bbd783f803d2663: function(arg0, arg1) {
            getObject(arg0).depthCompare = __wbindgen_enum_GpuCompareFunction[arg1];
        },
        __wbg_set_depth_load_op_a1b09a7ef6e4c3d4: function(arg0, arg1) {
            getObject(arg0).depthLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
        },
        __wbg_set_depth_read_only_ad84a1b622c0a941: function(arg0, arg1) {
            getObject(arg0).depthReadOnly = arg1 !== 0;
        },
        __wbg_set_depth_stencil_attachment_d17d58f9212bc033: function(arg0, arg1) {
            getObject(arg0).depthStencilAttachment = getObject(arg1);
        },
        __wbg_set_depth_stencil_feb50d4de7e35eb1: function(arg0, arg1) {
            getObject(arg0).depthStencil = getObject(arg1);
        },
        __wbg_set_depth_store_op_da4447aff70c723d: function(arg0, arg1) {
            getObject(arg0).depthStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
        },
        __wbg_set_depth_write_enabled_88916d207b076ce9: function(arg0, arg1) {
            getObject(arg0).depthWriteEnabled = arg1 !== 0;
        },
        __wbg_set_device_53f6c9076263d322: function(arg0, arg1) {
            getObject(arg0).device = getObject(arg1);
        },
        __wbg_set_dimension_21a837bba6fdd54d: function(arg0, arg1) {
            getObject(arg0).dimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
        },
        __wbg_set_distanceModel_5664671fd8e00b15: function(arg0, arg1) {
            getObject(arg0).distanceModel = __wbindgen_enum_DistanceModelType[arg1];
        },
        __wbg_set_download_327e0fbad75893f0: function(arg0, arg1, arg2) {
            getObject(arg0).download = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_dst_factor_cfb1cd4ae3e673d9: function(arg0, arg1) {
            getObject(arg0).dstFactor = __wbindgen_enum_GpuBlendFactor[arg1];
        },
        __wbg_set_end_of_pass_write_index_11a1206efb355fa2: function(arg0, arg1) {
            getObject(arg0).endOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_end_of_pass_write_index_4481029dd8b47829: function(arg0, arg1) {
            getObject(arg0).endOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_entries_7c51a3643c761ecf: function(arg0, arg1, arg2) {
            getObject(arg0).entries = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_entries_cc80db0de96ea4ff: function(arg0, arg1, arg2) {
            getObject(arg0).entries = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_entry_point_3725bb995522ec92: function(arg0, arg1, arg2) {
            getObject(arg0).entryPoint = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_entry_point_a7593f073d0352f7: function(arg0, arg1, arg2) {
            getObject(arg0).entryPoint = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_entry_point_edfb9dd004493bc8: function(arg0, arg1, arg2) {
            getObject(arg0).entryPoint = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_fillStyle_16593c82bd7127ca: function(arg0, arg1, arg2) {
            getObject(arg0).fillStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_fillStyle_1a796925e818ac7c: function(arg0, arg1) {
            getObject(arg0).fillStyle = getObject(arg1);
        },
        __wbg_set_fillStyle_a2961b4d44e572af: function(arg0, arg1, arg2) {
            getObject(arg0).fillStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_font_1f60a05a2544a2ff: function(arg0, arg1, arg2) {
            getObject(arg0).font = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_font_7ef69121e02464cd: function(arg0, arg1, arg2) {
            getObject(arg0).font = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_format_036434fd3db220e7: function(arg0, arg1) {
            getObject(arg0).format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_502a9d6bb7a509e3: function(arg0, arg1) {
            getObject(arg0).format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_67f2d982476ea95a: function(arg0, arg1) {
            getObject(arg0).format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_6bbbd4df6747733e: function(arg0, arg1) {
            getObject(arg0).format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_d1ce90a1fc3dd2cc: function(arg0, arg1) {
            getObject(arg0).format = __wbindgen_enum_GpuVertexFormat[arg1];
        },
        __wbg_set_format_d4c8ef73b087dab0: function(arg0, arg1) {
            getObject(arg0).format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_d7ddba819243bb4b: function(arg0, arg1) {
            getObject(arg0).format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_fragment_9bc5cc98c7c59fef: function(arg0, arg1) {
            getObject(arg0).fragment = getObject(arg1);
        },
        __wbg_set_front_face_8f4065ccd6ca4a56: function(arg0, arg1) {
            getObject(arg0).frontFace = __wbindgen_enum_GpuFrontFace[arg1];
        },
        __wbg_set_g_ac69babc4a0b2338: function(arg0, arg1) {
            getObject(arg0).g = arg1;
        },
        __wbg_set_globalAlpha_ae4b85201dda64c5: function(arg0, arg1) {
            getObject(arg0).globalAlpha = arg1;
        },
        __wbg_set_globalCompositeOperation_8d071d7528280f24: function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).globalCompositeOperation = getStringFromWasm0(arg1, arg2);
        }, arguments); },
        __wbg_set_height_c9789c1c77eaedff: function(arg0, arg1) {
            getObject(arg0).height = arg1 >>> 0;
        },
        __wbg_set_href_f7219c6cbf6bedc3: function(arg0, arg1, arg2) {
            getObject(arg0).href = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_id_60955e6018d03b26: function(arg0, arg1, arg2) {
            getObject(arg0).id = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_imageSmoothingEnabled_eb22252b69819404: function(arg0, arg1) {
            getObject(arg0).imageSmoothingEnabled = arg1 !== 0;
        },
        __wbg_set_innerHTML_7af59a832a09a074: function(arg0, arg1, arg2) {
            getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_47dd4e012a516db2: function(arg0, arg1, arg2) {
            getObject(arg0).label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_4efe866cd2283a4b: function(arg0, arg1, arg2) {
            getObject(arg0).label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_61e330b0149970d7: function(arg0, arg1, arg2) {
            getObject(arg0).label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_9d0387d0cd51b8d1: function(arg0, arg1, arg2) {
            getObject(arg0).label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_b4778e2a702d7fb8: function(arg0, arg1, arg2) {
            getObject(arg0).label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_bc05c8a3d707e4e5: function(arg0, arg1, arg2) {
            getObject(arg0).label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_d4d9152bd0968b75: function(arg0, arg1, arg2) {
            getObject(arg0).label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_f5bc1e93a198b23f: function(arg0, arg1, arg2) {
            getObject(arg0).label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_fa12c564a9136d01: function(arg0, arg1, arg2) {
            getObject(arg0).label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_layout_28120b0d39fbd2f4: function(arg0, arg1) {
            getObject(arg0).layout = getObject(arg1);
        },
        __wbg_set_layout_4e86f7c1671a06d0: function(arg0, arg1) {
            getObject(arg0).layout = getObject(arg1);
        },
        __wbg_set_layout_f2b50454b73e9111: function(arg0, arg1) {
            getObject(arg0).layout = getObject(arg1);
        },
        __wbg_set_lineCap_c9b7bdd3b7f5ce74: function(arg0, arg1, arg2) {
            getObject(arg0).lineCap = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_lineJoin_c1780f2096e6f970: function(arg0, arg1, arg2) {
            getObject(arg0).lineJoin = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_lineWidth_cc15473552c60c9c: function(arg0, arg1) {
            getObject(arg0).lineWidth = arg1;
        },
        __wbg_set_load_op_18ff54287ff40151: function(arg0, arg1) {
            getObject(arg0).loadOp = __wbindgen_enum_GpuLoadOp[arg1];
        },
        __wbg_set_loop_05cd640845793b88: function(arg0, arg1) {
            getObject(arg0).loop = arg1 !== 0;
        },
        __wbg_set_loop_a2c9bac20caf74e0: function(arg0, arg1) {
            getObject(arg0).loop = arg1 !== 0;
        },
        __wbg_set_mag_filter_91c348b29e1a6e57: function(arg0, arg1) {
            getObject(arg0).magFilter = __wbindgen_enum_GpuFilterMode[arg1];
        },
        __wbg_set_maxDistance_8f07202ef788f2dd: function(arg0, arg1) {
            getObject(arg0).maxDistance = arg1;
        },
        __wbg_set_max_anisotropy_9021c8be62116522: function(arg0, arg1) {
            getObject(arg0).maxAnisotropy = arg1;
        },
        __wbg_set_method_dcb32343247ec427: function(arg0, arg1, arg2) {
            getObject(arg0).method = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_min_filter_fdbe96994784b13c: function(arg0, arg1) {
            getObject(arg0).minFilter = __wbindgen_enum_GpuFilterMode[arg1];
        },
        __wbg_set_mip_level_count_0fa4c89090a80173: function(arg0, arg1) {
            getObject(arg0).mipLevelCount = arg1 >>> 0;
        },
        __wbg_set_mip_level_count_45273154f3074fa8: function(arg0, arg1) {
            getObject(arg0).mipLevelCount = arg1 >>> 0;
        },
        __wbg_set_mipmap_filter_6257bee513be4590: function(arg0, arg1) {
            getObject(arg0).mipmapFilter = __wbindgen_enum_GpuMipmapFilterMode[arg1];
        },
        __wbg_set_module_168054323d22d57d: function(arg0, arg1) {
            getObject(arg0).module = getObject(arg1);
        },
        __wbg_set_module_1ad4e7343951303d: function(arg0, arg1) {
            getObject(arg0).module = getObject(arg1);
        },
        __wbg_set_module_37e4693f4015166d: function(arg0, arg1) {
            getObject(arg0).module = getObject(arg1);
        },
        __wbg_set_multisample_ac97d83121c016de: function(arg0, arg1) {
            getObject(arg0).multisample = getObject(arg1);
        },
        __wbg_set_multisampled_edcd0bd60842a75d: function(arg0, arg1) {
            getObject(arg0).multisampled = arg1 !== 0;
        },
        __wbg_set_offset_618dd5b4444a70c9: function(arg0, arg1) {
            getObject(arg0).offset = arg1 >>> 0;
        },
        __wbg_set_offset_d3396f8f19d42b80: function(arg0, arg1) {
            getObject(arg0).offset = arg1 >>> 0;
        },
        __wbg_set_onchange_448456cb6fbbd47d: function(arg0, arg1) {
            getObject(arg0).onchange = getObject(arg1);
        },
        __wbg_set_onclick_c978dc4919c3c885: function(arg0, arg1) {
            getObject(arg0).onclick = getObject(arg1);
        },
        __wbg_set_onended_a562c0d904e528ee: function(arg0, arg1) {
            getObject(arg0).onended = getObject(arg1);
        },
        __wbg_set_onerror_24dddc0e8dc54601: function(arg0, arg1) {
            getObject(arg0).onerror = getObject(arg1);
        },
        __wbg_set_onload_a85e3f5ed3002dde: function(arg0, arg1) {
            getObject(arg0).onload = getObject(arg1);
        },
        __wbg_set_onsuccess_e970bffc8d0e7520: function(arg0, arg1) {
            getObject(arg0).onsuccess = getObject(arg1);
        },
        __wbg_set_onuncapturederror_cb90b60ddf08d90e: function(arg0, arg1) {
            getObject(arg0).onuncapturederror = getObject(arg1);
        },
        __wbg_set_onupgradeneeded_f9d42df70899331b: function(arg0, arg1) {
            getObject(arg0).onupgradeneeded = getObject(arg1);
        },
        __wbg_set_operation_d8894b0f9aa3db27: function(arg0, arg1) {
            getObject(arg0).operation = __wbindgen_enum_GpuBlendOperation[arg1];
        },
        __wbg_set_oversample_069342d1d837df37: function(arg0, arg1) {
            getObject(arg0).oversample = __wbindgen_enum_OverSampleType[arg1];
        },
        __wbg_set_panningModel_326afd220189674a: function(arg0, arg1) {
            getObject(arg0).panningModel = __wbindgen_enum_PanningModelType[arg1];
        },
        __wbg_set_passive_31e9f895d2ac0046: function(arg0, arg1) {
            getObject(arg0).passive = arg1 !== 0;
        },
        __wbg_set_power_preference_5a61738a9603767f: function(arg0, arg1) {
            getObject(arg0).powerPreference = __wbindgen_enum_GpuPowerPreference[arg1];
        },
        __wbg_set_preload_ccedc73ebf51475a: function(arg0, arg1, arg2) {
            getObject(arg0).preload = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_primitive_3b6c52880715b903: function(arg0, arg1) {
            getObject(arg0).primitive = getObject(arg1);
        },
        __wbg_set_query_set_00e2fc0da42f1581: function(arg0, arg1) {
            getObject(arg0).querySet = getObject(arg1);
        },
        __wbg_set_query_set_589a18812331e156: function(arg0, arg1) {
            getObject(arg0).querySet = getObject(arg1);
        },
        __wbg_set_r_ccf9fe3e4d993555: function(arg0, arg1) {
            getObject(arg0).r = arg1;
        },
        __wbg_set_refDistance_69243dd2454d0031: function(arg0, arg1) {
            getObject(arg0).refDistance = arg1;
        },
        __wbg_set_required_features_1555498f978e929a: function(arg0, arg1, arg2) {
            getObject(arg0).requiredFeatures = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_resolve_target_gpu_texture_view_3f66a54e8fb729d5: function(arg0, arg1) {
            getObject(arg0).resolveTarget = getObject(arg1);
        },
        __wbg_set_resource_3367d3d5c5114a05: function(arg0, arg1) {
            getObject(arg0).resource = getObject(arg1);
        },
        __wbg_set_resource_gpu_buffer_0183c6050c20bcec: function(arg0, arg1) {
            getObject(arg0).resource = getObject(arg1);
        },
        __wbg_set_resource_gpu_buffer_binding_c689b0dc9379a99b: function(arg0, arg1) {
            getObject(arg0).resource = getObject(arg1);
        },
        __wbg_set_resource_gpu_texture_view_2ed0c241a5831597: function(arg0, arg1) {
            getObject(arg0).resource = getObject(arg1);
        },
        __wbg_set_rolloffFactor_ad35dc23f46d4a3a: function(arg0, arg1) {
            getObject(arg0).rolloffFactor = arg1;
        },
        __wbg_set_rows_per_image_8287d6c20450d5dc: function(arg0, arg1) {
            getObject(arg0).rowsPerImage = arg1 >>> 0;
        },
        __wbg_set_rows_per_image_97b4a26f53457ed3: function(arg0, arg1) {
            getObject(arg0).rowsPerImage = arg1 >>> 0;
        },
        __wbg_set_sample_count_06bd5decdb9d2471: function(arg0, arg1) {
            getObject(arg0).sampleCount = arg1 >>> 0;
        },
        __wbg_set_sample_type_1e16dd736b699077: function(arg0, arg1) {
            getObject(arg0).sampleType = __wbindgen_enum_GpuTextureSampleType[arg1];
        },
        __wbg_set_sampler_e7e837391d5b7c70: function(arg0, arg1) {
            getObject(arg0).sampler = getObject(arg1);
        },
        __wbg_set_shader_location_c7d2a36952c56851: function(arg0, arg1) {
            getObject(arg0).shaderLocation = arg1 >>> 0;
        },
        __wbg_set_shadowBlur_216915d92b179fbe: function(arg0, arg1) {
            getObject(arg0).shadowBlur = arg1;
        },
        __wbg_set_shadowColor_0e2bd5d3400b0d3b: function(arg0, arg1, arg2) {
            getObject(arg0).shadowColor = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_shadowOffsetX_0dbcf4ca8295e0c3: function(arg0, arg1) {
            getObject(arg0).shadowOffsetX = arg1;
        },
        __wbg_set_shadowOffsetY_de495fcc5a5ae809: function(arg0, arg1) {
            getObject(arg0).shadowOffsetY = arg1;
        },
        __wbg_set_size_4716cf40ee7a64be: function(arg0, arg1) {
            getObject(arg0).size = arg1 >>> 0;
        },
        __wbg_set_size_c2eb75980b76e0ac: function(arg0, arg1, arg2) {
            getObject(arg0).size = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_size_cde62ab9f3c247b5: function(arg0, arg1) {
            getObject(arg0).size = arg1 >>> 0;
        },
        __wbg_set_source_3ffc7e2ebf9b8967: function(arg0, arg1) {
            getObject(arg0).source = getObject(arg1);
        },
        __wbg_set_src_e0fe5ef5d5c648aa: function(arg0, arg1, arg2) {
            getObject(arg0).src = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_src_factor_b29ea819b1b5d6f6: function(arg0, arg1) {
            getObject(arg0).srcFactor = __wbindgen_enum_GpuBlendFactor[arg1];
        },
        __wbg_set_storage_texture_8a09df33e1fc957b: function(arg0, arg1) {
            getObject(arg0).storageTexture = getObject(arg1);
        },
        __wbg_set_store_op_6d2fb6cc3e66341e: function(arg0, arg1) {
            getObject(arg0).storeOp = __wbindgen_enum_GpuStoreOp[arg1];
        },
        __wbg_set_strokeStyle_d51608fa918b53d4: function(arg0, arg1, arg2) {
            getObject(arg0).strokeStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_targets_0328c60fcfc0f564: function(arg0, arg1, arg2) {
            getObject(arg0).targets = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_textAlign_1bda1733d57574ed: function(arg0, arg1, arg2) {
            getObject(arg0).textAlign = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textAlign_4c67cfad430a7166: function(arg0, arg1, arg2) {
            getObject(arg0).textAlign = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textBaseline_7cf16a557c77ce8d: function(arg0, arg1, arg2) {
            getObject(arg0).textBaseline = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textBaseline_d5ba548751584f49: function(arg0, arg1, arg2) {
            getObject(arg0).textBaseline = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_texture_5e4b2303dfbef0fa: function(arg0, arg1) {
            getObject(arg0).texture = getObject(arg1);
        },
        __wbg_set_texture_9f67e0e92f7c2af3: function(arg0, arg1) {
            getObject(arg0).texture = getObject(arg1);
        },
        __wbg_set_texture_e6c25e637d8a26a4: function(arg0, arg1) {
            getObject(arg0).texture = getObject(arg1);
        },
        __wbg_set_timestamp_writes_7c6dc68dd71b8302: function(arg0, arg1) {
            getObject(arg0).timestampWrites = getObject(arg1);
        },
        __wbg_set_timestamp_writes_e6b1b4ec02a89a80: function(arg0, arg1) {
            getObject(arg0).timestampWrites = getObject(arg1);
        },
        __wbg_set_topology_36f1b64a3f922a23: function(arg0, arg1) {
            getObject(arg0).topology = __wbindgen_enum_GpuPrimitiveTopology[arg1];
        },
        __wbg_set_type_19b6e3a56c3fa1f1: function(arg0, arg1, arg2) {
            getObject(arg0).type = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_type_593ebcdf56243aae: function(arg0, arg1) {
            getObject(arg0).type = __wbindgen_enum_GpuQueryType[arg1];
        },
        __wbg_set_type_67234415b0e1a64a: function(arg0, arg1) {
            getObject(arg0).type = __wbindgen_enum_BiquadFilterType[arg1];
        },
        __wbg_set_type_8e9e3a274f528f3e: function(arg0, arg1) {
            getObject(arg0).type = __wbindgen_enum_GpuSamplerBindingType[arg1];
        },
        __wbg_set_type_dcf75b2afa60c694: function(arg0, arg1, arg2) {
            getObject(arg0).type = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_type_ff9e8f3110ad0cbf: function(arg0, arg1) {
            getObject(arg0).type = __wbindgen_enum_GpuBufferBindingType[arg1];
        },
        __wbg_set_usage_7177d2b5a0cb7ed1: function(arg0, arg1) {
            getObject(arg0).usage = arg1 >>> 0;
        },
        __wbg_set_usage_aad5221dc614bcd1: function(arg0, arg1) {
            getObject(arg0).usage = arg1 >>> 0;
        },
        __wbg_set_value_bf45a2dfcba57147: function(arg0, arg1) {
            getObject(arg0).value = arg1;
        },
        __wbg_set_vertex_5b13839ffaff5f37: function(arg0, arg1) {
            getObject(arg0).vertex = getObject(arg1);
        },
        __wbg_set_view_dimension_07b5c66946c78442: function(arg0, arg1) {
            getObject(arg0).viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
        },
        __wbg_set_view_formats_3a90e9d7b267f004: function(arg0, arg1, arg2) {
            getObject(arg0).viewFormats = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_view_gpu_texture_view_8d8dd7c0c832a505: function(arg0, arg1) {
            getObject(arg0).view = getObject(arg1);
        },
        __wbg_set_view_gpu_texture_view_9f7613e516713508: function(arg0, arg1) {
            getObject(arg0).view = getObject(arg1);
        },
        __wbg_set_visibility_78cfb96740c9deb5: function(arg0, arg1) {
            getObject(arg0).visibility = arg1 >>> 0;
        },
        __wbg_set_width_b0e1267db4b196b5: function(arg0, arg1) {
            getObject(arg0).width = arg1 >>> 0;
        },
        __wbg_set_write_mask_99f4985fd8dc4442: function(arg0, arg1) {
            getObject(arg0).writeMask = arg1 >>> 0;
        },
        __wbg_shiftKey_be6d5295ee392b33: function(arg0) {
            const ret = getObject(arg0).shiftKey;
            return ret;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_start_12f5fef12bb396ee: function() { return handleError(function (arg0) {
            getObject(arg0).start();
        }, arguments); },
        __wbg_start_a96bdad03de1993e: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).start(arg1 >>> 0);
            return ret;
        }, arguments); },
        __wbg_state_54551b9345fb007f: function(arg0) {
            const ret = getObject(arg0).state;
            return (__wbindgen_enum_AudioContextState.indexOf(ret) + 1 || 4) - 1;
        },
        __wbg_static_accessor_GLOBAL_266715b9d96ba635: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_10fb7dc1ae063179: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_SELF_0b583911f537483a: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_WINDOW_d7f903d1508cbdc4: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_status_ce0a98d3c57125f3: function(arg0) {
            const ret = getObject(arg0).status;
            return ret;
        },
        __wbg_stop_5bb124fd4e4d0ad6: function() { return handleError(function (arg0) {
            getObject(arg0).stop();
        }, arguments); },
        __wbg_stop_cb3ffac6e0d1547a: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).stop(arg1);
        }, arguments); },
        __wbg_stringify_52ff602c1cc4fbb6: function() { return handleError(function (arg0) {
            const ret = JSON.stringify(getObject(arg0));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_stroke_385d731098398489: function(arg0) {
            getObject(arg0).stroke();
        },
        __wbg_style_4bce24230e493a7c: function(arg0) {
            const ret = getObject(arg0).style;
            return addHeapObject(ret);
        },
        __wbg_submit_46ecaa46ea244579: function(arg0, arg1, arg2) {
            getObject(arg0).submit(getArrayJsValueViewFromWasm0(arg1, arg2));
        },
        __wbg_suspend_2fbcec238306d5b7: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).suspend();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_tagName_e83500df63beeee1: function(arg0, arg1) {
            const ret = getObject(arg1).tagName;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_target_38ae9feb025b820c: function(arg0) {
            const ret = getObject(arg0).target;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_then_c949d5a25a4e78f8: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        },
        __wbg_then_e71170d78fcf8954: function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_toBlob_47074fb2ab6d463d: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).toBlob(getObject(arg1));
        }, arguments); },
        __wbg_toDataURL_d31e9b09748ee59d: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).toDataURL();
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_toISOString_3217c36cf55f719b: function(arg0) {
            const ret = getObject(arg0).toISOString();
            return addHeapObject(ret);
        },
        __wbg_top_ff4627294d2cdeb8: function(arg0) {
            const ret = getObject(arg0).top;
            return ret;
        },
        __wbg_transaction_55a3a4e0134a0fd6: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).transaction(getStringFromWasm0(arg1, arg2), __wbindgen_enum_IdbTransactionMode[arg3]);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_transferToImageBitmap_c2dae3e64c90a2a8: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).transferToImageBitmap();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_type_4167a7757cb08c4e: function(arg0) {
            const ret = getObject(arg0).type;
            return (__wbindgen_enum_GpuCompilationMessageType.indexOf(ret) + 1 || 4) - 1;
        },
        __wbg_unmap_692c7e866688f956: function(arg0) {
            getObject(arg0).unmap();
        },
        __wbg_userAgent_cde4c65cc98db425: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).userAgent;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_value_b0f8eddb3809690f: function(arg0) {
            const ret = getObject(arg0).value;
            return ret;
        },
        __wbg_warn_13abc63e0d4b3527: function(arg0) {
            console.warn(getObject(arg0));
        },
        __wbg_width_3d0dce3d9892e35e: function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        },
        __wbg_width_8a9cc26363f10d5f: function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        },
        __wbg_width_d5e379bde85b6eaa: function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        },
        __wbg_width_e95ad291d52fe17e: function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        },
        __wbg_width_f1234333517222c4: function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        },
        __wbg_writeBuffer_2c02702ce0fa25c3: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).writeBuffer(getObject(arg1), arg2 >>> 0, getArrayU8FromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_writeText_79a2b37e525e9424: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).writeText(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        },
        __wbg_writeTexture_2bebcfd79754afa7: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).writeTexture(getObject(arg1), getArrayU8FromWasm0(arg2, arg3), getObject(arg4), getArrayJsValueViewFromWasm0(arg5, arg6));
        }, arguments); },
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 39, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1071);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 43, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1075);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [F64], shim_idx: 46, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1079);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 39, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1071_5);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("FocusEvent")], shim_idx: 39, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1071_6);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("GPUCompilationInfo")], shim_idx: 43, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1075_7);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000007: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("GPUDevice")], shim_idx: 43, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1075_8);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000008: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("GPUUncapturedErrorEvent")], shim_idx: 39, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1071_9);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000009: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("KeyboardEvent")], shim_idx: 39, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1071_10);
            return addHeapObject(ret);
        },
        __wbindgen_generic_000000000000000a: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 39, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1071_11);
            return addHeapObject(ret);
        },
        __wbindgen_generic_000000000000000b: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("PointerEvent")], shim_idx: 39, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1071_12);
            return addHeapObject(ret);
        },
        __wbindgen_generic_000000000000000c: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("WheelEvent")], shim_idx: 39, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1071_13);
            return addHeapObject(ret);
        },
        __wbindgen_generic_000000000000000d: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("any")], shim_idx: 43, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1075_14);
            return addHeapObject(ret);
        },
        __wbindgen_generic_000000000000000e: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("undefined")], shim_idx: 43, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1075_15);
            return addHeapObject(ret);
        },
        __wbindgen_generic_000000000000000f: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [String, Externref], shim_idx: 41, ret: Externref, inner_ret: Some(Externref) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1073);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000010: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 61, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_2051);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000011: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000012: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./game_bg.js": import0,
    };
}

const lAudioContext = (typeof AudioContext !== 'undefined' ? AudioContext : (typeof webkitAudioContext !== 'undefined' ? webkitAudioContext : undefined));
function __wasm_bindgen_func_elem_2051(arg0, arg1) {
    wasm.__wasm_bindgen_func_elem_2051(arg0, arg1);
}

function __wasm_bindgen_func_elem_1071(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1071(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_1071_5(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1071_5(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_1071_6(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1071_6(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_1071_9(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1071_9(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_1071_10(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1071_10(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_1071_11(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1071_11(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_1071_12(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1071_12(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_1071_13(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1071_13(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_3393(arg0, arg1, arg2, arg3) {
    wasm.__wasm_bindgen_func_elem_3393(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

function __wasm_bindgen_func_elem_1075(arg0, arg1, arg2) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.__wasm_bindgen_func_elem_1075(retptr, arg0, arg1, addHeapObject(arg2));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

function __wasm_bindgen_func_elem_1075_7(arg0, arg1, arg2) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.__wasm_bindgen_func_elem_1075_7(retptr, arg0, arg1, addHeapObject(arg2));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

function __wasm_bindgen_func_elem_1075_8(arg0, arg1, arg2) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.__wasm_bindgen_func_elem_1075_8(retptr, arg0, arg1, addHeapObject(arg2));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

function __wasm_bindgen_func_elem_1075_14(arg0, arg1, arg2) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.__wasm_bindgen_func_elem_1075_14(retptr, arg0, arg1, addHeapObject(arg2));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

function __wasm_bindgen_func_elem_1075_15(arg0, arg1, arg2) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.__wasm_bindgen_func_elem_1075_15(retptr, arg0, arg1, addHeapObject(arg2));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

function __wasm_bindgen_func_elem_1073(arg0, arg1, arg2, arg3) {
    const ptr0 = passStringToWasm0(arg2, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.__wasm_bindgen_func_elem_1073(arg0, arg1, ptr0, len0, addHeapObject(arg3));
    return takeObject(ret);
}

function __wasm_bindgen_func_elem_1079(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1079(arg0, arg1, arg2);
}


const __wbindgen_enum_AudioContextState = ["suspended", "running", "closed"];


const __wbindgen_enum_BiquadFilterType = ["lowpass", "highpass", "bandpass", "lowshelf", "highshelf", "peaking", "notch", "allpass"];


const __wbindgen_enum_DistanceModelType = ["linear", "inverse", "exponential"];


const __wbindgen_enum_GpuAddressMode = ["clamp-to-edge", "repeat", "mirror-repeat"];


const __wbindgen_enum_GpuBlendFactor = ["zero", "one", "src", "one-minus-src", "src-alpha", "one-minus-src-alpha", "dst", "one-minus-dst", "dst-alpha", "one-minus-dst-alpha", "src-alpha-saturated", "constant", "one-minus-constant", "src1", "one-minus-src1", "src1-alpha", "one-minus-src1-alpha"];


const __wbindgen_enum_GpuBlendOperation = ["add", "subtract", "reverse-subtract", "min", "max"];


const __wbindgen_enum_GpuBufferBindingType = ["uniform", "storage", "read-only-storage"];


const __wbindgen_enum_GpuCanvasAlphaMode = ["opaque", "premultiplied"];


const __wbindgen_enum_GpuCompareFunction = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];


const __wbindgen_enum_GpuCompilationMessageType = ["error", "warning", "info"];


const __wbindgen_enum_GpuCullMode = ["none", "front", "back"];


const __wbindgen_enum_GpuErrorFilter = ["validation", "out-of-memory", "internal"];


const __wbindgen_enum_GpuFilterMode = ["nearest", "linear"];


const __wbindgen_enum_GpuFrontFace = ["ccw", "cw"];


const __wbindgen_enum_GpuIndexFormat = ["uint16", "uint32"];


const __wbindgen_enum_GpuLoadOp = ["load", "clear"];


const __wbindgen_enum_GpuMipmapFilterMode = ["nearest", "linear"];


const __wbindgen_enum_GpuPowerPreference = ["low-power", "high-performance"];


const __wbindgen_enum_GpuPrimitiveTopology = ["point-list", "line-list", "line-strip", "triangle-list", "triangle-strip"];


const __wbindgen_enum_GpuQueryType = ["occlusion", "timestamp"];


const __wbindgen_enum_GpuSamplerBindingType = ["filtering", "non-filtering", "comparison"];


const __wbindgen_enum_GpuStorageTextureAccess = ["write-only", "read-only", "read-write"];


const __wbindgen_enum_GpuStoreOp = ["store", "discard"];


const __wbindgen_enum_GpuTextureFormat = ["r8unorm", "r8snorm", "r8uint", "r8sint", "r16unorm", "r16snorm", "r16uint", "r16sint", "r16float", "rg8unorm", "rg8snorm", "rg8uint", "rg8sint", "r32uint", "r32sint", "r32float", "rg16unorm", "rg16snorm", "rg16uint", "rg16sint", "rg16float", "rgba8unorm", "rgba8unorm-srgb", "rgba8snorm", "rgba8uint", "rgba8sint", "bgra8unorm", "bgra8unorm-srgb", "rgb9e5ufloat", "rgb10a2uint", "rgb10a2unorm", "rg11b10ufloat", "rg32uint", "rg32sint", "rg32float", "rgba16unorm", "rgba16snorm", "rgba16uint", "rgba16sint", "rgba16float", "rgba32uint", "rgba32sint", "rgba32float", "stencil8", "depth16unorm", "depth24plus", "depth24plus-stencil8", "depth32float", "depth32float-stencil8", "bc1-rgba-unorm", "bc1-rgba-unorm-srgb", "bc2-rgba-unorm", "bc2-rgba-unorm-srgb", "bc3-rgba-unorm", "bc3-rgba-unorm-srgb", "bc4-r-unorm", "bc4-r-snorm", "bc5-rg-unorm", "bc5-rg-snorm", "bc6h-rgb-ufloat", "bc6h-rgb-float", "bc7-rgba-unorm", "bc7-rgba-unorm-srgb", "etc2-rgb8unorm", "etc2-rgb8unorm-srgb", "etc2-rgb8a1unorm", "etc2-rgb8a1unorm-srgb", "etc2-rgba8unorm", "etc2-rgba8unorm-srgb", "eac-r11unorm", "eac-r11snorm", "eac-rg11unorm", "eac-rg11snorm", "astc-4x4-unorm", "astc-4x4-unorm-srgb", "astc-5x4-unorm", "astc-5x4-unorm-srgb", "astc-5x5-unorm", "astc-5x5-unorm-srgb", "astc-6x5-unorm", "astc-6x5-unorm-srgb", "astc-6x6-unorm", "astc-6x6-unorm-srgb", "astc-8x5-unorm", "astc-8x5-unorm-srgb", "astc-8x6-unorm", "astc-8x6-unorm-srgb", "astc-8x8-unorm", "astc-8x8-unorm-srgb", "astc-10x5-unorm", "astc-10x5-unorm-srgb", "astc-10x6-unorm", "astc-10x6-unorm-srgb", "astc-10x8-unorm", "astc-10x8-unorm-srgb", "astc-10x10-unorm", "astc-10x10-unorm-srgb", "astc-12x10-unorm", "astc-12x10-unorm-srgb", "astc-12x12-unorm", "astc-12x12-unorm-srgb"];


const __wbindgen_enum_GpuTextureSampleType = ["float", "unfilterable-float", "depth", "sint", "uint"];


const __wbindgen_enum_GpuTextureViewDimension = ["1d", "2d", "2d-array", "cube", "cube-array", "3d"];


const __wbindgen_enum_GpuVertexFormat = ["uint8", "uint8x2", "uint8x4", "sint8", "sint8x2", "sint8x4", "unorm8", "unorm8x2", "unorm8x4", "snorm8", "snorm8x2", "snorm8x4", "uint16", "uint16x2", "uint16x4", "sint16", "sint16x2", "sint16x4", "unorm16", "unorm16x2", "unorm16x4", "snorm16", "snorm16x2", "snorm16x4", "float16", "float16x2", "float16x4", "float32", "float32x2", "float32x3", "float32x4", "uint32", "uint32x2", "uint32x3", "uint32x4", "sint32", "sint32x2", "sint32x3", "sint32x4", "unorm10-10-10-2", "unorm8x4-bgra"];


const __wbindgen_enum_IdbTransactionMode = ["readonly", "readwrite", "versionchange", "readwriteflush", "cleanup"];


const __wbindgen_enum_OverSampleType = ["none", "2x", "4x"];


const __wbindgen_enum_PanningModelType = ["equalpower", "HRTF"];

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_export5(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayJsValueViewFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(getObject(mem.getUint32(i, true)));
    }
    return result;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getClampedArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ClampedArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedUint8ClampedArrayMemory0 = null;
function getUint8ClampedArrayMemory0() {
    if (cachedUint8ClampedArrayMemory0 === null || cachedUint8ClampedArrayMemory0.byteLength === 0) {
        cachedUint8ClampedArrayMemory0 = new Uint8ClampedArray(wasm.memory.buffer);
    }
    return cachedUint8ClampedArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export3(addHeapObject(e));
    }
}

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_export5(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArrayF32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4, 4) >>> 0;
    getFloat32ArrayMemory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    cachedUint8ClampedArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('game_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
