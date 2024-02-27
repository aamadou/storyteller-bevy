import {type InspectorChanged} from "@storyteller/studio";

document.addEventListener('inspector-changed', (event) => {
    // Cast the event to the InspectorDomChanged type to access its details in TypeScript
    const customEvent = event as InspectorChanged;

    const data = customEvent['detail'];

    //print to console
    console.log(data);
    
});