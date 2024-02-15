import { register_new_change, type InspectorDomChanged } from "@storyteller/studio";

console.log("Hello, World!");


function unpack_vnode(
    vnode: any
) {
    if (vnode["Text"] != undefined) {
        var text_element = document.createTextNode(vnode["Text"]);
        return text_element;
    } else if (vnode["Component"] != undefined) {
        var node_data = vnode["Component"];
        var element = document.createElement(node_data["tag"]);
        //apply attributes
        for (var i = 0; i < node_data["attributes"].length; i++) {
            var attr = node_data["attributes"][i];
            element.setAttribute(attr[0], attr[1]);
        }
        if (node_data["id"] != undefined) {
            element.setAttribute("id", node_data["id"]);
        };
        //apply children
        if (node_data["children"] != undefined) {
            for (var j = 0; j < node_data["children"].length; j++) {
                var child = node_data["children"][j];
                var child_element = unpack_vnode(child);
                element.appendChild(child_element);
            }
        }
        //add onchange hook if input
        if (node_data["onchange"] == true) {
            element.oninput = (event: Event) => {
                var input = event.target as HTMLInputElement;
                const id = input.getAttribute("id");
                if (id != null) {
                    console.log("onchange", id, input.value);
                    const value = input.value;
                    register_new_change(id, value);
                }
            }            
        }
        return element;
    }
}

function apply_muts(
    event : InspectorDomChanged
) {
    
    if (event.detail !== null) {

        var lit_root = document.getElementById("inspector");
        var shadowroot = lit_root?.shadowRoot;
        var root = shadowroot?.getElementById("inspector");
        if (root) {
            //remove all children
            
        } else {
            console.log("root not found");
            return;
        }

        var muts = JSON.parse(event.detail);

        if (muts.length > 0) {
            console.log("muts", muts);
        }
        
        for (var i = 0; i < muts.length; i++) {
            var mut = muts[i];
            if (mut["ReplaceAll"] != undefined) {
                var vnode = mut["ReplaceAll"];
                var element = unpack_vnode(vnode);
                while (root.firstChild) {
                    root.removeChild(root.firstChild);
                }
                root.appendChild(element);
            } else if (mut["SetAttibute"] != undefined) {
                let m = mut["SetAttibute"];
                let id = m["id"];
                let value = m["value"];
                let attribute = m["attribute"];
                let element = shadowroot?.getElementById(id);
                if (!element) {
                    console.log("element not found", id);
                } else {
                    if (attribute == "value") {
                        var input = element as HTMLInputElement;
                        if (input) {
                            input.value = value;
                            //update input value
                            // Create a new 'input' event
                            const event = new Event('input', {
                                bubbles: true,
                                cancelable: true,
                            });

                            // Dispatch the event
                            input.dispatchEvent(event);

                            console.log("input value updated", value);
                            continue;
                        }
                    }
                    element.setAttribute(attribute, value);
                }
            }
        }

    } else {
        console.log('Selection has been cleared.');
    }
}

document.addEventListener('inspector-dom-changed', (event: CustomEvent) => {
    // Cast the event to the InspectorDomChanged type to access its details in TypeScript
    const customEvent = event as InspectorDomChanged;
    // Now you can access the detail property, which is either a string or null
    apply_muts(customEvent);
});

