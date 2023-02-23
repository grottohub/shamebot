export type Guild = {
    id: number,
    name: string,
    icon?: string,
    send_to?: number,
}

export type User = {
    id: number,
    username: string,
    discriminator: string,
    avatar_hash: string,
}

export type List = {
    id: string,
    title: string,
    user_id: number,
}

export type Task = {
    id: string,
    list_id: string,
    user_id: number,
    guild_id: number,
    title: string,
    content?: string,
    checked: boolean,
    pester?: number,
    due_at?: number,
    proof_id?: string,
    pester_job?: string,
    overdue_job?: string,
    reminder_job?: string,
}

export type Proof = {
    id: string,
    content?: string,
    image?: string,
    approved: boolean,
}

export enum RequestStatus {
    Accepted = "accepted",
    Pending = "pending",
    Rejected = "rejected",
}

export type AccountabilityRequest = {
    requesting_user: number,
    requested_user: number,
    task_id: string,
    status: RequestStatus,
}

export type Token = {
    id: string,
    access_token: string,
    token_type: string,
    expires_at: number,
    refresh_token: string,
    scope: string,
}

export type ApiKey = {
    user_id: number,
    discord_token: string,
    key: string,
}
