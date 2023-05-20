/*
 Generated by typeshare 1.4.0
*/

export interface PublicConfig {
	redirect_to_first_oauth_provider: boolean;
	oauth_providers: string[];
}

export interface CreateLocation {
	type_?: string;
	name: string;
	description?: string;
	requirements?: string;
}

export interface Location {
	id: number;
	user_id: number;
	type_?: string;
	name: string;
	description?: string;
	requirements?: string;
	created_on: string;
	updated_at: string;
}

export interface UpdateLocation {
	type_?: string;
	name?: string;
	description?: string;
	requirements?: string;
}

export interface CreateUser {
	email: string;
	phone_number?: string;
	fname: string;
	lname: string;
	password: string;
}

export interface LoginData {
	email: string;
	password: string;
}

export enum PermissionLevel {
	Student = "Student",
	Teacher = "Teacher",
	Admin = "Admin",
}

export interface LocalLoginData {
	updated_at: string;
}

export interface OAuthLoginData {
	provider: string;
	associated_email: string;
	updated_at: string;
}

export interface UserData {
	email: string;
	email_verified: boolean;
	phone_number?: string;
	fname: string;
	lname: string;
	bio?: string;
	profile_image?: string;
	permission_level: PermissionLevel;
	joined_on: string;
	updated_at: string;
	last_login?: string;
	local_login?: LocalLoginData;
	oauth_providers: OAuthLoginData[];
}

export interface OAuthErrorMessage {
	error_msg: string;
}

