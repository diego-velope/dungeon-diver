/**
 * Shared namespace for modular TV PAL (Dungeon Diver).
 */
(function (global) {
    'use strict';

    var PLATFORM_IDS = {
        TIZEN: 'tizen',
        WEBOS: 'webos',
        VIZIO: 'vizio',
        FIRETV: 'firetv',
        ANDROID_TV: 'android_tv',
        BROWSER: 'browser'
    };

    global.DungeonDiverPAL = global.DungeonDiverPAL || {};
    global.DungeonDiverPAL.PLATFORM_IDS = PLATFORM_IDS;
    global.DungeonDiverPAL.platforms = global.DungeonDiverPAL.platforms || {};

    global.DungeonDiverPAL.registerPlatform = function (id, impl) {
        if (!impl || !impl.keyMapping) {
            console.warn('[DungeonDiverPAL] registerPlatform: missing keyMapping for', id);
            return;
        }
        global.DungeonDiverPAL.platforms[id] = impl;
    };
})(typeof window !== 'undefined' ? window : this);
