export function get(): Promise<any> {
    return fetch("http://127.0.0.1:8081/config")
        .then(response => response.json());
}

export function post(data: any): Promise<any> {
    return fetch("http://127.0.0.1:8081/config", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify(data)
    })
    .then(response => response.json());
}