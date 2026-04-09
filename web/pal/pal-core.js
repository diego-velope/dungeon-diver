/**
 * TV Platform Abstraction Layer (PAL) core for Dungeon Diver.
 */
(function () {
    'use strict';

    var TDP = window.DungeonDiverPAL;
    var P = TDP.PLATFORM_IDS;

    var currentPlatform = null;
    var keyMapping = null;
    var isInitialized = false;
    var debugMode = true;

    function debugLog() {
        if (debugMode) {
            console.log.apply(console, arguments);
        }
    }

    function detectPlatform() {
        return TDP.detectPlatform();
    }

    function mapKeycodeToAction(keyCode) {
        if (!keyMapping) return null;
        for (var action in keyMapping) {
            if (!Object.prototype.hasOwnProperty.call(keyMapping, action)) continue;
            var codes = keyMapping[action];
            if (codes.indexOf(keyCode) !== -1) {
                return action;
            }
        }
        return null;
    }

    function forwardToRust(action, pressed) {
        var functionMap = {
            up: 'mq_handle_up',
            down: 'mq_handle_down',
            left: 'mq_handle_left',
            right: 'mq_handle_right',
            action: 'mq_handle_action',
            back: 'mq_handle_back'
        };

        var funcName = functionMap[action];
        if (!funcName) return;

        if (typeof window[funcName] === 'function') {
            window[funcName](pressed ? 1 : 0);
        } else if (debugMode) {
            console.warn('[PAL] Function not available:', funcName);
        }
    }

    function isBackKeyCode(keyCode) {
        var backCodes = keyMapping && keyMapping.back ? keyMapping.back : [10009, 4, 27];
        return backCodes.indexOf(keyCode) !== -1;
    }

    function handleBackEvent(e, pressed) {
        var keyCode = e.keyCode || e.which;
        if (!isBackKeyCode(keyCode)) return false;
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        forwardToRust('back', pressed);
        return true;
    }

    function handleKeyDown(e) {
        var keyCode = e.keyCode || e.which;
        var action = mapKeycodeToAction(keyCode);
        if (!action) return true;
        if (action === 'back') {
            handleBackEvent(e, true);
        } else {
            forwardToRust(action, true);
            e.preventDefault();
            e.stopPropagation();
            e.stopImmediatePropagation();
        }
        return false;
    }

    function handleKeyUp(e) {
        var keyCode = e.keyCode || e.which;
        var action = mapKeycodeToAction(keyCode);
        if (!action) return true;
        if (action === 'back') {
            handleBackEvent(e, false);
        } else {
            forwardToRust(action, false);
            e.preventDefault();
            e.stopPropagation();
            e.stopImmediatePropagation();
        }
        return false;
    }

    function init(options) {
        options = options || {};
        if (isInitialized) return;

        debugMode = options.debug || false;
        currentPlatform = detectPlatform();
        var impl = TDP.platforms[currentPlatform] || TDP.platforms[P.BROWSER];
        if (!impl) return;

        keyMapping = impl.keyMapping;
        if (typeof impl.registerKeys === 'function') {
            impl.registerKeys({ currentPlatform: currentPlatform });
        }

        window.addEventListener('keydown', handleKeyDown, true);
        window.addEventListener('keyup', handleKeyUp, true);
        window.addEventListener(
            'keydown',
            function (e) {
                if (handleBackEvent(e, true)) return false;
            },
            true
        );

        window.addEventListener('contextmenu', function (e) {
            e.preventDefault();
            e.stopPropagation();
            return false;
        });

        isInitialized = true;
    }

    function shutdown() {
        if (isInitialized) {
            window.removeEventListener('keydown', handleKeyDown, true);
            window.removeEventListener('keyup', handleKeyUp, true);
            isInitialized = false;
        }

        var impl = currentPlatform && (TDP.platforms[currentPlatform] || TDP.platforms[P.BROWSER]);
        if (impl && typeof impl.shutdownHost === 'function') {
            try {
                var hostHandledExit = impl.shutdownHost() === true;
                if (hostHandledExit) return false;
            } catch (e) {
                console.warn('[PAL] shutdownHost failed:', e);
                return true;
            }
        }
        return true;
    }

    function _handleAndroidKeyEvent(keyCode, state) {
        var webKeyCode = keyCode;
        if (keyCode === 66) webKeyCode = 13;
        if (keyCode === 82) webKeyCode = 999;
        var action = mapKeycodeToAction(webKeyCode);
        if (action) {
            forwardToRust(action, state === 'down');
        }
    }

    var TV_PAL = {
        init: init,
        shutdown: shutdown,
        getPlatform: function () {
            return currentPlatform;
        },
        setDebug: function (enabled) {
            debugMode = enabled;
        },
        _handleAndroidKeyEvent: _handleAndroidKeyEvent
    };

    window._handleAndroidKeyEvent = function (keyCode, state) {
        if (TV_PAL && TV_PAL._handleAndroidKeyEvent) {
            TV_PAL._handleAndroidKeyEvent(keyCode, state);
        }
    };

    var tvPalDebug = new URLSearchParams(window.location.search).has('debug');
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function () {
            TV_PAL.init({ debug: tvPalDebug });
        });
    } else {
        TV_PAL.init({ debug: tvPalDebug });
    }

    window.TV_PAL = TV_PAL;
})();
