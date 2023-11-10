/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface ConnectOptions {
  spillPath?: string
  disableTls?: boolean
  cloudAddr?: string
  location?: string
  storageOptions?: Record<string, string>
}
/** Connect to a GlareDB database. */
export function connect(dataDirOrCloudUrl?: string | undefined | null, options?: ConnectOptions | undefined | null): Promise<Connection>
export interface JsRecordBatch {
  schema: JsSchema
  columns: Array<JsDynArray>
  rowCount: number
}
export interface JsDynArray {
  datatype: DataTypeKind
  nulls?: Array<number>
  offsets?: Array<number>
  values: Array<number>
}
export interface JsArrowArray {
  datatype: DataTypeKind
  data: object
}
export interface JsDataType {
  kind: DataTypeKind
  inner?: any
}
export interface JsSchema {
  fields: Array<JsField>
  metadata: Record<string, string>
}
export interface JsField {
  name: string
  dataType: DataTypeKind
  nullable: boolean
  dictId?: number
  dictIsOrdered?: boolean
  metadata: Record<string, string>
}
export const enum DataTypeKind {
  Null = 'Null',
  Boolean = 'Boolean',
  Int8 = 'Int8',
  Int16 = 'Int16',
  Int32 = 'Int32',
  Int64 = 'Int64',
  UInt8 = 'UInt8',
  UInt16 = 'UInt16',
  UInt32 = 'UInt32',
  UInt64 = 'UInt64',
  Float16 = 'Float16',
  Float32 = 'Float32',
  Float64 = 'Float64',
  Timestamp = 'Timestamp',
  Date32 = 'Date32',
  Date64 = 'Date64',
  Time32 = 'Time32',
  Time64 = 'Time64',
  Duration = 'Duration',
  Interval = 'Interval',
  Binary = 'Binary',
  FixedSizeBinary = 'FixedSizeBinary',
  LargeBinary = 'LargeBinary',
  Utf8 = 'Utf8',
  LargeUtf8 = 'LargeUtf8',
  List = 'List',
  FixedSizeList = 'FixedSizeList',
  LargeList = 'LargeList',
  Struct = 'Struct',
  Union = 'Union',
  Dictionary = 'Dictionary',
  Decimal128 = 'Decimal128',
  Decimal256 = 'Decimal256',
  Map = 'Map',
  RunEndEncoded = 'RunEndEncoded'
}
/** A connected session to a GlareDB database. */
export class Connection {
  /**
   * Returns a default connection to an in-memory database.
   *
   * The database is only initialized once, and all subsequent calls will
   * return the same connection.
   */
  static defaultInMemory(): Promise<Connection>
  /**
   * Run a SQL operation against a GlareDB database.
   *
   * All operations that write or modify data are executed
   * directly, but all query operations run lazily when you process
   * their results with `show`, `toArrow`, or
   * `toPolars`, or call the `execute` method.
   *
   * # Examples
   *
   * Show the output of a query.
   *
   * ```javascript
   * import glaredb from "@glaredb/node"
   *
   * let con = glaredb.connect()
   * let cursor = await con.sql('select 1');
   * await cursor.show()
   * ```
   *
   * Convert the output of a query to a Pandas dataframe.
   *
   * ```javascript
   * import glaredb from "@glaredb/node"
   *
   * let con = glaredb.connect()
   * ```
   *
   * Execute the query to completion, returning no output. This is useful
   * when the query output doesn't matter, for example, creating a table or
   * inserting data into a table.
   *
   * ```javascript
   * import glaredb from "@glaredb/node"
   *
   * con = glaredb.connect()
   * await con.sql('create table my_table (a int)').then(cursor => cursor.execute())
   * ```
   */
  sql(query: string): Promise<JsLogicalPlan>
  /**
   * Run a PRQL query against a GlareDB database. Does not change
   * the state or dialect of the connection object.
   *
   * ```javascript
   * import glaredb from "@glaredb/node"
   *
   * let con = glaredb.connect()
   * let cursor = await con.sql('from my_table | take 1');
   * await cursor.show()
   * ```
   *
   * All operations execute lazily when their results are
   * processed.
   */
  prql(query: string): Promise<JsLogicalPlan>
  /**
   * Execute a query.
   *
   * # Examples
   *
   * Creating a table.
   *
   * ```js
   * import glaredb from "@glaredb/node"
   *
   * con = glaredb.connect()
   * con.execute('create table my_table (a int)')
   * ```
   */
  execute(query: string): Promise<void>
  /** Close the current session. */
  close(): Promise<void>
}
export class JsLogicalPlan {
  toString(): string
  show(): Promise<void>
  execute(): Promise<void>
  recordBatches(): Promise<Array<JsRecordBatch>>
  toIpc(): Promise<Buffer>
  /**
   * Convert to a Polars DataFrame.
   * "nodejs-polars" must be installed as a peer dependency.
   * See https://www.npmjs.com/package/nodejs-polars
   */
  toPolars(): pl.DataFrame
  /**
   * Convert to an "apache-arrow" Table.
   * "apache-arrow" must be installed as a peer dependency.
   * See https://www.npmjs.com/package/apache-arrow
   */
  toArrow(): arrow.Table<any>
}
