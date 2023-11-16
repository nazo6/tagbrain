/* generated using openapi-typescript-codegen -- do no edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { ScanRequest } from '../models/ScanRequest';

import type { CancelablePromise } from '../core/CancelablePromise';
import type { BaseHttpRequest } from '../core/BaseHttpRequest';

export class ApiService {

    constructor(public readonly httpRequest: BaseHttpRequest) {}

    /**
     * @param requestBody
     * @returns any Scan file
     * @throws ApiError
     */
    public scan(
        requestBody: ScanRequest,
    ): CancelablePromise<any> {
        return this.httpRequest.request({
            method: 'POST',
            url: '/api/scan',
            body: requestBody,
            mediaType: 'application/json',
            errors: {
                400: `Invalid request`,
                500: `Failed to send job`,
            },
        });
    }

    /**
     * @returns any Scan all files in source folder
     * @throws ApiError
     */
    public scanAll(): CancelablePromise<Array<any>> {
        return this.httpRequest.request({
            method: 'POST',
            url: '/api/scan_all',
            errors: {
                500: `Failed to send job`,
            },
        });
    }

}
