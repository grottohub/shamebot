// API Client for shamebot's backend
type GenericError = {
    message: string,
}

type GenericResponse = {
    status: number,
    data: Array<any>,
    error?: GenericError,
}

type Health = {
    status: number;
    text: string;
}

export class Client {
    base_url: string;

    constructor(url: string) {
        this.base_url = url;
    }

    async request(endpoint: string): Promise<GenericResponse> {
        let resp = await fetch(`${this.base_url}/${endpoint}`, { mode: 'cors' });

        return resp.json();
    }

    async health(): Promise<Health> {
        let resp = await fetch(`${this.base_url}/health`, { mode: 'cors' });

        return {
            status: resp.status,
            text: resp.statusText,
        };
    }
}
