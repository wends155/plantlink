import MqttInNode from './MqttInNode.svelte';
import ModbusReadNode from './ModbusReadNode.svelte';
import RhaiFunctionNode from './RhaiFunctionNode.svelte';
import InjectNode from './InjectNode.svelte';
import ConsoleNode from './ConsoleNode.svelte';
import NatsBrokerNode from './NatsBrokerNode.svelte';
import NatsSubNode from './NatsSubNode.svelte';
import NatsPubNode from './NatsPubNode.svelte';

const registry = {
  'mqtt-in': MqttInNode,
  'modbus-read': ModbusReadNode,
  'rhai-function': RhaiFunctionNode,
  inject: InjectNode,
  console: ConsoleNode,
  'nats-broker': NatsBrokerNode,
  'nats-sub': NatsSubNode,
  'nats-pub': NatsPubNode
};

export const registerNode = (type, component) => {
  console.log(`Registering node: ${type}`);
  registry[type] = component;
};

export const getNodeTypes = () => registry;
