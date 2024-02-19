import { on } from "@storyteller/framework";
import { type InspectorDomChanged } from "dist/studio/studio";
import { LitElement, html, unsafeCSS, nothing, HTMLTemplateResult } from "lit";
import { customElement, state } from "lit/decorators.js";
import { property } from "lodash";


@customElement("sts-inspector")
export class AppElement extends LitElement {
    @state() vdom?: string | null;
    @state() generation = 0;

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
        
        
    }

}