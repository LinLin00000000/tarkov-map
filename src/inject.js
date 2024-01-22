
/* global spp */
document.addEventListener('DOMContentLoaded', () => {
    if (!window._injected) {
        window._injected = true;

        const __TAURI__ = window.__TAURI__
        // const dispatch = useDispatch();
        // const spp = (x, z) => dispatch(setPlayerPosition({ position: { x, z } }));

        __TAURI__?.event.listen('gps', (event) => {
            const data = event.payload;
            console.log('Received message:', data);
            const matches = data.match(/-?\d+\.\d+/g);

            if (matches && matches.length >= 3) {
                const x = parseFloat(matches[0]);
                const z = parseFloat(matches[2]);

                spp(x, z);
            } else {
                console.log('匹配失败');
            }
        });
        // const host = '127.0.0.1';
        // const port = 7543;

        // const eventSource = new EventSource(`http://${host}:${port}/map`);
        // eventSource.onmessage = function (event) {
        //     const data = event.data;
        //     console.log('Received message:', data);
        //     const matches = data.match(/-?\d+\.\d+/g);

        //     if (matches && matches.length >= 3) {
        //         const x = parseFloat(matches[0]);
        //         const z = parseFloat(matches[2]);

        //         spp(x, z);
        //     } else {
        //         console.log('匹配失败');
        //     }
        // };
    }
});

// dynamicQuery.ts
var dynamicQuery = /* @__PURE__ */ (() => {
    function addObserver(target, callback) {
        let canceled = false;
        const observer = new MutationObserver((mutations) => {
            for (const mutation of mutations) {
                if (mutation.type === 'childList' || mutation.type === 'attributes') {
                    if (canceled) return;
                    callback(mutation.target);
                    for (const node of mutation.addedNodes) {
                        if (canceled) return;
                        callback(node);
                    }
                }
            }
        });
        observer.observe(target, {
            subtree: true,
            childList: true,
            attributes: true,
        });
        return () => {
            canceled = true;
            observer.disconnect();
        };
    }
    const observedNodeMap = /* @__PURE__ */ new WeakMap();
    function addProcessor(target, processor) {
        let observedNode = observedNodeMap.get(target);
        if (!observedNode) {
            let checked = /* @__PURE__ */ new WeakSet();
            let processors = /* @__PURE__ */ new Set();
            const checkAndApply = (e) => {
                if (checked && !checked.has(e)) {
                    checked.add(e);
                    processors?.forEach(([s, f]) => {
                        if (e.matches(s)) {
                            f(e);
                        }
                    });
                }
            };
            const disconnect = addObserver(target, (e) => {
                if (e instanceof Element) {
                    checkAndApply(e);
                    e.querySelectorAll('*').forEach(checkAndApply);
                }
            });
            observedNode = {
                processors,
                remove: () => {
                    disconnect();
                    checked = null;
                    processors = null;
                },
            };
            observedNodeMap.set(target, observedNode);
        }
        observedNode.processors.add(processor);
    }
    function removeProcessor(target, processor) {
        const observedNode = observedNodeMap.get(target);
        if (!observedNode) return false;
        const isDeleteInThisTime = observedNode.processors.delete(processor);
        if (!observedNode.processors.size) {
            observedNode.remove();
            observedNodeMap.delete(target);
        }
        return isDeleteInThisTime;
    }
    return function (selector, callback = console.log, options = {}) {
        const {
            parent = document,
            once = true,
            timeout = -1,
            onTimeout = () => console.log('dynamicQuery Timeout!', arguments),
            all = true,
            allDelay = 1e3,
        } = options;
        const selectors = Array.isArray(selector) ? selector : [selector];
        const notExistSelectors = selectors.filter((selector2) => {
            const result = all
                ? parent.querySelectorAll(selector2)
                : [parent.querySelector(selector2)].filter((e) => e !== null);
            result.forEach(callback);
            return result.length === 0;
        });
        if (once && notExistSelectors.length === 0) return () => false;
        const listenSelectors = once ? notExistSelectors : selectors;
        const processors = listenSelectors.map((selector2) => {
            const processed = /* @__PURE__ */ new WeakSet();
            let timer;
            const process = (e) => {
                if (!processed.has(e)) {
                    processed.add(e);
                    callback(e);
                    if (once) {
                        if (all) {
                            clearTimeout(timer);
                            timer = setTimeout(remove, allDelay);
                        } else {
                            remove();
                        }
                    }
                }
            };
            const processor = [selector2, process];
            const remove = () => removeProcessor(parent, processor);
            addProcessor(parent, processor);
            return remove;
        });
        const removeAllProcessor = () => processors.every((f) => f());
        if (timeout >= 0) {
            setTimeout(() => {
                removeAllProcessor();
                onTimeout();
            }, timeout);
        }
        return removeAllProcessor;
    };
})();
