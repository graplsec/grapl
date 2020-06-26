import * as cdk from "@aws-cdk/core";
import * as dynamodb from "@aws-cdk/aws-dynamodb";

import { Service } from "./service";
import { RemovalPolicy } from "@aws-cdk/core";

import { Watchful } from "./vendor/cdk-watchful/lib/watchful";

function create_table(
    scope: cdk.Construct,
    name: string,
) {
    return new dynamodb.Table(scope, name, {
        tableName: name,
        partitionKey: {
            name: 'pseudo_key',
            type: dynamodb.AttributeType.STRING
        },
        sortKey: {
            name: 'create_time',
            type: dynamodb.AttributeType.NUMBER
        },
        billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        removalPolicy: RemovalPolicy.DESTROY,
    });
}

export class HistoryDb extends cdk.Construct {

    readonly proc_history: dynamodb.Table;
    readonly file_history: dynamodb.Table;
    readonly outbound_connection_history: dynamodb.Table;
    readonly inbound_connection_history: dynamodb.Table;
    readonly network_connection_history: dynamodb.Table;
    readonly ip_connection_history: dynamodb.Table;
    readonly asset_history: dynamodb.Table;
    readonly node_id_retry_table: dynamodb.Table;
    readonly dynamic_session_table: dynamodb.Table;
    readonly static_mapping_table: dynamodb.Table;

    constructor(scope: cdk.Construct, id: string, watchful: Watchful) {
        super(scope, id);

        this.proc_history = create_table(this, 'process_history_table');
        // watchful.watchDynamoTable(this.proc_history.tableName, this.proc_history);

        this.file_history = create_table(this, 'file_history_table');
        // watchful.watchDynamoTable(this.file_history.tableName, this.file_history);

        this.outbound_connection_history = create_table(this, 'outbound_connection_history_table');
        // watchful.watchDynamoTable(this.outbound_connection_history.tableName, this.outbound_connection_history);

        this.inbound_connection_history = create_table(this, 'inbound_connection_history_table');
        // watchful.watchDynamoTable(this.inbound_connection_history.tableName, this.inbound_connection_history);

        this.network_connection_history = create_table(this, 'network_connection_history_table');
        // watchful.watchDynamoTable(this.network_connection_history.tableName, this.network_connection_history);

        this.ip_connection_history = create_table(this, 'ip_connection_history_table');
        // watchful.watchDynamoTable(this.ip_connection_history.tableName, this.ip_connection_history);

        this.dynamic_session_table = create_table(this, 'dynamic_session_table');
        // watchful.watchDynamoTable(this.dynamic_session_table.tableName, this.dynamic_session_table);

        this.asset_history = new dynamodb.Table(this, 'asset_id_mappings', {
            tableName: "asset_id_mappings",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            sortKey: {
                name: 'c_timestamp',
                type: dynamodb.AttributeType.NUMBER
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });
        // watchful.watchDynamoTable(this.asset_history.tableName, this.asset_history);

        this.static_mapping_table = new dynamodb.Table(this, 'static_mapping_table', {
            tableName: "static_mapping_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });
        // watchful.watchDynamoTable(this.static_mapping_table.tableName, this.static_mapping_table);

        this.node_id_retry_table = new dynamodb.Table(this, 'node_id_retry_table', {
            tableName: "node_id_retry_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            timeToLiveAttribute: "ttl_ts",
            removalPolicy: RemovalPolicy.DESTROY,
        });
        // watchful.watchDynamoTable(this.node_id_retry_table.tableName, this.node_id_retry_table);
    }

    allowReadWrite(service: Service) {
        this.proc_history.grantReadWriteData(service.event_handler);
        this.file_history.grantReadWriteData(service.event_handler);
        this.outbound_connection_history.grantReadWriteData(service.event_handler);
        this.inbound_connection_history.grantReadWriteData(service.event_handler);
        this.network_connection_history.grantReadWriteData(service.event_handler);
        this.ip_connection_history.grantReadWriteData(service.event_handler);
        this.asset_history.grantReadWriteData(service.event_handler);
        this.node_id_retry_table.grantReadWriteData(service.event_handler);
        this.static_mapping_table.grantReadWriteData(service.event_handler);
        this.dynamic_session_table.grantReadWriteData(service.event_handler);

        this.proc_history.grantReadWriteData(service.event_retry_handler);
        this.file_history.grantReadWriteData(service.event_retry_handler);
        this.outbound_connection_history.grantReadWriteData(service.event_retry_handler);
        this.inbound_connection_history.grantReadWriteData(service.event_retry_handler);
        this.network_connection_history.grantReadWriteData(service.event_retry_handler);
        this.ip_connection_history.grantReadWriteData(service.event_retry_handler);
        this.asset_history.grantReadWriteData(service.event_retry_handler);
        this.node_id_retry_table.grantReadWriteData(service.event_retry_handler);
        this.static_mapping_table.grantReadWriteData(service.event_retry_handler);
        this.dynamic_session_table.grantReadWriteData(service.event_retry_handler);
    }
}
