export interface EndpointExample {
    method: string;
    path: string;
    description: string;
    parameters?: Array<{
        name: string;
        type: string;
        required: boolean;
        description: string;
    }>;
    response: string;
    accessLevel: string;
}

export interface ModuleDocumentation {
    name: string;
    displayName: string;
    description: string;
    category: string;
    endpoints: EndpointExample[];
}
