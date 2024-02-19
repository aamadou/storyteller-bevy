import { on } from "@storyteller/framework";
import { type InspectorDomChanged } from "dist/studio/studio";
import { LitElement, html, unsafeCSS, nothing, HTMLTemplateResult } from "lit";
import { customElement, state } from "lit/decorators.js";
import { property } from "lodash";

//vdom example
/// {
///"subtree":0,
/// "dirty_scopes":[],
/// "templates":[{"name":"studio\\src\\ui\\inspector.rs:46:15:160899","roots":[{"type":"Element","tag":"div","namespace":null,"attrs":[],"children":[{"type":"Text","text":"Inspector"},{"type":"Element","tag":"div","namespace":null,"attrs":[],"children":[{"type":"Text","text":"hello world"}]}]}],"node_paths":[],"attr_paths":[]}],
/// "edits":[{"type":"LoadTemplate","name":"studio\\src\\ui\\inspector.rs:46:15:160899","index":0,"id":1},{"type":"AppendChildren","id":0,"m":1}]}


@customElement("sts-inspector")
export class AppElement extends LitElement {
    @state() vdom?: string | null;
    @state() generation = 0;
    @state() vdom_elements?: VdomElement[] | null;

    // @on("window:inspector-dom-changed")
	// onInspectorDomChanged({ detail: state}: InspectorDomChanged): void {
	// 	console.log("inspector dom changed:", state);
    //     //change vdom and activate render
    //     this.vdom = null;
    //     this.vdom = state;
    //     this.generation += 1;

    //     // Activate the render function
    //     this.requestUpdate();
	// }

    protected override render() { 

        this.shadowRoot

        return html`

            <style>
                .inspector {
                    overflow: auto;
                
                    box-sizing: border-box;
                    width: 290px;
                    max-height: calc(100vh - 32px);
                    padding: 10px 15px;
                
                    color: #FFFC;
                    background: #0006;
                    backdrop-filter: blur(16px);
                    border: 8px solid transparent;
                    border-radius: 0.5rem;
                
                    pointer-events: auto;
                
                    &::-webkit-scrollbar {
                        width: 8px;
                        height: 8px;
                
                        &-thumb {
                            border-radius: 2px;
                            background: #FFF2;
                
                            &:hover {
                                background: #FFF3;
                            }
                        }
                    }

                    font-family: "Fira Sans", sans-serif;
                    font-size: 14px;
                }
            </style>

            <div id="inspector"></div>
        `

        //convert string vdom to json object
        // if (this.vdom) {
        //     var vdom = JSON.parse(this.vdom);

        //     var templates = vdom["templates"]; 
        //     var edits = vdom["edits"];

        //     console.log("edits", edits);
        //     console.log("templates", templates);

        //     var element_stack = [];

        //     var element_tree = [];

        //     //loop through edits
        //     for (const edit of edits) {
                
        //         var type = edit["type"];
        //         if (type == "LoadTemplate") {
        //             var index = edit["index"];
        //             var template_name = edit["name"];
        //             var id = edit["id"];

        //             var target_root = null;
        //             for (const template of templates) {
        //                 if (template["name"] == template_name) {
        //                     target_root = template["roots"][index];
        //                     break;
        //                 }
        //             }
        //             if (target_root == null) {
        //                 console.log("template not found:", template_name);
        //                 continue;
        //             } else {
        //                 target_root["id"] = id;
        //                 element_stack.push(target_root);
        //             }
        //         } else if (type == "AppendChildren") {
        //             var id = edit["id"];
        //             var count = edit["m"];
        //             if (id == 0) {
        //                 for (var i = 0; i < count; i++) {
        //                     var current_element = element_stack.pop();
        //                     element_tree.push(current_element);
        //                 }
        //             } else {
        //                 //TODO
        //             }
        //         } else if (type == "HydrateText") {
        //             var child_path = edit["path"];
        //             var target_node = element_stack[element_stack.length - 1];

        //             for (const path of child_path) {
        //                 target_node = target_node["children"][path];
        //             }

        //             //add text node to target_node
        //             if (target_node["children"] == undefined) {
        //                 target_node["children"] = [];
        //             }
        //             target_node["children"].push({type: "Text", text: edit["value"], id: edit["id"]});


        //         }
        //     }

        //     console.log("element tree", element_tree);

        //     //convert any to VdomElement in element_tree
        //     var vdom_elements = element_tree.map((element) => {
        //         return element as VdomElement;
        //     });

        //     this.vdom_elements = vdom_elements;

        //     return html`
            
        //     <style>
        //         .inspector {
        //             overflow: auto;
                
        //             box-sizing: border-box;
        //             width: 290px;
        //             max-height: calc(100vh - 32px);
        //             padding: 10px 15px;
                
        //             color: #FFFC;
        //             background: #0006;
        //             backdrop-filter: blur(16px);
        //             border: 8px solid transparent;
        //             border-radius: 0.5rem;
                
        //             pointer-events: auto;
                
        //             &::-webkit-scrollbar {
        //                 width: 8px;
        //                 height: 8px;
                
        //                 &-thumb {
        //                     border-radius: 2px;
        //                     background: #FFF2;
                
        //                     &:hover {
        //                         background: #FFF3;
        //                     }
        //                 }
        //             }
        //         }
        //     </style>
            
        //     <div id="inspector" class="inspector">
        //         ${this.vdom_elements.map((element, index) => html`<sts-inspector-element generation=${this.generation} .vdom=${element} key=${index}></sts-inspector-element>`)}
        //     </div>`

           
        // } else {
        //     //For each vdom element render InspectorElement
        //     return html` 
                
        //     `
        // }
        
        
    }

}

export interface VdomElement {
    id?: number;
    type?: string;
    text?: string;
    tag?: string;
    namespace?: string;
    attrs?: any[];
    children?: VdomElement[];
}


/// Element for recursive render vdom tree
@customElement("sts-inspector-element")
export class InspectorElement extends LitElement {
    @property({ type: Object, attribute: true })
    vdom: VdomElement;

    @property({ type: Number, attribute: true })
    generation: number;

    protected override render() {
        var tag = this.vdom["tag"];
        var type = this.vdom.type;
        
        if (type == "Text") {
            return html`<div>${this.vdom["text"]}</div>`
        } else if (type == "Dynamic") {
            return html`
                <div id=${this.vdom["id"]}>
                    ${this.vdom["children"].map((element) => html`<sts-inspector-element .vdom=${element} .generation=${this.generation}></sts-inspector-element>`)}
                </div>
            `
        }

        if (tag == "div") {
            return html`
            <style>
                .tabber {
                    padding: 0 0 0 10px;
                }
            </style>
            
            <div id=${this.vdom["id"]} class="tabber">
                ${this.vdom["children"].map((element) => html`<sts-inspector-element .vdom=${element} .generation=${this.generation}></sts-inspector-element>`)}
            </div>`
        } else {
            return html`<div>${this.vdom["text"]}</div>`
        }
    }
}