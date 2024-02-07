import { type InspectorDomChanged } from "dist/studio/studio";

console.log("Hello, World!");


function load_child(stack : any[], child_path : number[]) {
    // iterate through each number and get that child
    var node : any = stack[stack.length - 1];
    for (const ptr of child_path) {
        var steps = ptr;
        var m = 0;
        for (node = node.firstChild; m < steps; m++) {
            node = node.nextSibling;
        }
    }
    return node;
}

function laod_template(edit : any, stack : any[]) {
    
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
        var templates = vdom["templates"]; 
        var edits = vdom["edits"];

        console.log("edits", edits);
        console.log("templates", templates);

        var element_stack = [];
        var nodes : any = {}

        

        nodes[0] = document.createElement("div");
        root.appendChild(nodes[0]);
        element_stack.push(root);

        //loop through edits
        for (const edit of edits) {
            
            var type = edit["type"];
            if (type == "LoadTemplate") {
                // var index = edit["index"];
                // var template_name = edit["name"];
                // var id = edit["id"];

                // var target_root = null;
                // for (const template of templates) {
                //     if (template["name"] == template_name) {
                //         target_root = template["roots"][index];
                //         break;
                //     }
                // }
                // if (target_root == null) {
                //     console.log("template not found:", template_name);
                //     continue;
                // } else {
                //     target_root["id"] = id;
                //     element_stack.push(target_root);
                // }

                var id = edit["id"];
                var index = edit["index"];
                var template_name = edit["name"];

                var template_node = null;
                for (const template of templates) {
                    if (template["name"] == template_name) {
                        template_node = template["roots"][index];
                        break;
                    }
                }

                if (template_node == null) {
                    console.log("template not found:", template_name);
                    continue;
                } else {
                    var node = recursive_render(null, template_node);
                    nodes[id] = node;
                    element_stack.push(node);
                }

            } else if (type == "AppendChildren") {
                // var id = edit["id"];
                // var count = edit["m"];
                // if (id == 0) {
                //     for (var i = 0; i < count; i++) {
                //         var current_element = element_stack.pop();
                //         element_tree.push(current_element);
                //     }
                // } else {
                //     //TODO
                // }

                var id = edit["id"];
                var many = edit["m"];

                var target_node = nodes[id];

                let append_nodes = element_stack.splice(element_stack.length - many);

                for (var i = 0; i < many; i++) {
                    target_node.appendChild(append_nodes[i]);
                }
            } else if (type == "HydrateText") {
                // var child_path = edit["path"];
                // var target_node = element_stack[element_stack.length - 1];

                // for (const path of child_path) {
                //     target_node = target_node["children"][path];
                // }

                // if (target_node == null) {
                //     console.log("target_node not found:", child_path);
                //     continue;
                // }

                // //add text node to target_node
                // if (target_node["children"] == undefined) {
                //     target_node["children"] = [];
                // }
                // target_node["children"].push({type: "Text", text: edit["value"], id: edit["id"]});

                var node : HTMLElement = load_child(element_stack, edit["path"]);
                if (node) {
                    node.textContent = edit["value"];
                } else {
                    console.log("node not found:", edit["path"]);
                }
            } else if (type == "ReplacePlaceholder") {
                var child_path = edit["path"];
                var manu = edit["m"];

                var els = [];
                for (var i = 0; i < manu; i++) {
                    var el = element_stack.pop();
                    els.push(el);
                }
                els.reverse();
                var target_node = load_child(element_stack, child_path);
                target_node.replaceWith(...els); 
                console.log("")
            } else if (type == "CreateTextNode") {
                var text_node : any = document.createTextNode(edit["value"]);
                nodes[edit["id"]] = text_node;
                element_stack.push(text_node);
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


export interface VdomElement {
    id?: number;
    type?: string;
    text?: string;
    tag?: string;
    namespace?: string;
    attrs?: any[];
    children?: VdomElement[];
}

function recursive_render(
    root: HTMLElement | null,
    element: VdomElement,
) {
    //create element
    var tag = element["tag"];
    if (element.type == "Text") {
        tag = "div";
    } else if (element.type == "Dynamic") {
        tag = "div";
    }
    
    var new_element = document.createElement(element["tag"] as string);

    //set text
    if (element.text != undefined) {
        new_element.textContent = element.text;
    }

    //set id
    if (element.id != undefined) {
        new_element.id = element.id.toString();
    }

    //set attrs
    if (element.attrs != undefined) {
        for (const attr of element.attrs) {
            var name = attr["name"];
            var value = attr["value"];
            new_element.setAttribute(name, value);
        }
    }

    //parent to root
    if (root != null) {
        root.appendChild(new_element);
    }

    //render children
    if (element["children"] != undefined) {
        for (const child of element["children"]) {
            recursive_render(new_element, child);
        }
    }

    return new_element;
}