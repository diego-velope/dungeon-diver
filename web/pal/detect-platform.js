/**
 * TV platform detection (user agent + host globals).
 * Depends on: pal/namespace.js
 */
(function (global) {
    'use strict';

    var P = global.DungeonDiverPAL.PLATFORM_IDS;

    function detectPlatform() {
        var rawUa = navigator.userAgent || '';
        var ua = rawUa.toLowerCase();

        console.log('[PAL] User Agent:', ua);

        if (global.tizen) {
            console.log('[PAL] Detected: Samsung Tizen');
            return P.TIZEN;
        }

        if (global.webOS) {
            console.log('[PAL] Detected: LG webOS');
            return P.WEBOS;
        }

        if (
            rawUa.search(/AmazonPlatform/i) > -1 ||
            rawUa.search(/AFTMM/i) > -1 ||
            rawUa.search(/AFTKM/i) > -1
        ) {
            console.log('[PAL] Detected: Amazon Fire TV (UA signature)');
            return P.FIRETV;
        }

        if (ua.includes('chromecast') || (ua.includes('android') && ua.includes('aarch64'))) {
            console.log('[PAL] Detected: Chromecast with Google TV');
            return P.ANDROID_TV;
        }

        if (
            ua.includes('aft') ||
            ua.includes('fire') ||
            ua.includes('silk') ||
            (ua.includes('android') && (ua.includes('tv') || ua.includes('aftn')))
        ) {
            console.log('[PAL] Detected: Amazon Fire TV / Android TV');
            return P.FIRETV;
        }

        if (ua.includes('android') && (ua.includes('tv') || ua.includes('nexus player'))) {
            console.log('[PAL] Detected: Android TV');
            return P.ANDROID_TV;
        }

        if (ua.includes('vizio')) {
            console.log('[PAL] Detected: Vizio');
            return P.VIZIO;
        }

        console.log('[PAL] Detected: Browser (desktop testing mode)');
        return P.BROWSER;
    }

    global.DungeonDiverPAL.detectPlatform = detectPlatform;
})(typeof window !== 'undefined' ? window : this);
