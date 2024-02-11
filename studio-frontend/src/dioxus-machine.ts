import { type InspectorDomChanged } from "dist/studio/studio";

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
        //apply children
        if (node_data["children"] != undefined) {
            for (var j = 0; j < node_data["children"].length; j++) {
                var child = node_data["children"][j];
                var child_element = unpack_vnode(child);
                element.appendChild(child_element);
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
        console.log("root", root);
        if (root) {
            //remove all children
            while (root.firstChild) {
                root.removeChild(root.firstChild);
            }
        } else {
            console.log("root not found");
            return;
        }

        var vdom = JSON.parse(event.detail);
        
        var element = unpack_vnode(vdom);
        root.appendChild(element);

    } else {
        console.log('Selection has been cleared.');
    }
}

document.addEventListener('inspector-dom-changed', (event: CustomEvent) => {
    // Cast the event to the InspectorDomChanged type to access its details in TypeScript
    const customEvent = event as InspectorDomChanged;
    console.log(customEvent);
    // Now you can access the detail property, which is either a string or null
    apply_muts(customEvent);
});

