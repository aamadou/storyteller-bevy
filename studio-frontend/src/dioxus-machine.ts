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

        //Create custom elements
        if (node_data["tag"] == "input" && element["type"] == "number") {
            create_float_input(element);
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
                append_style(root);

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

function create_float_input(element : HTMLInputElement) {

    console.log("create_float_input", element);

    let value = 0; // Начальное значение
    let isMouseDown = false;
    let startX = 0;

    element.addEventListener('mousedown', (e: MouseEvent) => {
        isMouseDown = true;
        startX = e.clientX;
        value = parseInt(element.value, 10);
        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
    });

    const onMouseMove = (e: MouseEvent) => {
        if (isMouseDown) {
            const dx = e.clientX - startX;
            startX = e.clientX;
            var float_value = parseFloat(element.value);
            if (float_value == null) {
                return;
            }
            element.value = (float_value + dx / 10.0).toString(); // Изменение на 1 за каждые 5 пикселей движения мыши
            //dispatch update event
            const event = new Event('input', {
                bubbles: true,
                cancelable: true,
            });
            element.dispatchEvent(event);
        }
    };

    const onMouseUp = () => {
        isMouseDown = false;
        document.removeEventListener('mousemove', onMouseMove);
        document.removeEventListener('mouseup', onMouseUp);
    };

    // element.addEventListener('keydown', (e: KeyboardEvent) => {
    //     if (!e.key.match(/^[0-9]$/)) {
    //         e.preventDefault(); // Предотвратить ввод нечисловых символов
    //     }
    // });
};

function append_style(parent : HTMLElement) {
    let style = document.createElement('style');

    // Устанавливаем тип содержимого для <style>
    style.type = 'text/css';

    // Добавляем CSS правила в элемент <style>. Используйте обратные кавычки для многострочного текста
    style.innerHTML = `
    input[type="number"] {
        background-color: #3333336e; /* Темный фон */
        color: #fff; /* Светлый текст */
        border: none; /* Убрать стандартную рамку */
        border-radius: 7px; /* Закругленные границы */
        padding: 5px; /* Внутренний отступ */
        outline: none; /* Убрать контур при фокусе */
    }
    input[type="number"]:focus {
        border: 2px solid #555; /* Стиль границы при фокусе */
        background-color: #222; /* Немного темнее фон при фокусе */
    }

    input[type="number"]::-webkit-inner-spin-button,
    input[type="number"]::-webkit-outer-spin-button {
        -webkit-appearance: none;
        margin: 0;
    }

    input[type="number"] {
        -moz-appearance: textfield;
    }
    
    .struct-grid {
        display: grid;
        grid-template-columns: min-content 1fr;
        column-gap: 8px;
        row-gap: 4px;
        align-items: center;
    }

    `;


    parent.appendChild(style);
}