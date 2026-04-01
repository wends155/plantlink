import InjectProperties from './InjectProperties.svelte';
import MqttInProperties from './MqttInProperties.svelte';
import NatsProperties from './NatsProperties.svelte';
import RhaiProperties from './RhaiProperties.svelte';

export const propertyComponents = {
  inject: InjectProperties,
  'mqtt-in': MqttInProperties,
  'nats-broker': NatsProperties,
  'nats-sub': NatsProperties,
  'nats-pub': NatsProperties,
  'rhai-function': RhaiProperties
};

export { InjectProperties, MqttInProperties, NatsProperties, RhaiProperties };
