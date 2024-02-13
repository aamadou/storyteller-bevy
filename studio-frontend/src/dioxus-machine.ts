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
            element.onchange = (event: Event) => {
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

