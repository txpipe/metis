# Create K8s cluster
kind create cluster --name supernode --config config.yml
kubectl config use-context kind-supernode 

# Install control plane
helm install control-plane ./control-plane/ \
  --namespace control-plane \
  --create-namespace

# Dolos
helm install dolos-preview ./dolos \
  --namespace dolos \
  --create-namespace \
  --set image.tag=v0.32.0 \
  --set extraLabels.supernode/status=ready \
  --set config.upstreamAddress=relay.cnode-m1.demeter.run:3002 \
  --set node.network=preview \
  --set node.networkMagic=2

# # Hydra
# kubectl create namespace hydra
# kubectl -n hydra create secret generic hydra-signing \
#  --from-file=hydra.sk=hydra.sk
# kubectl -n hydra create configmap hydra-verification \
#  --from-file=hydra.vk=hydra.vk
# helm install hydra ./hydra-node \
#  --namespace hydra \
#  --set keys.hydraSigning.existingSecret.name=hydra-signing \
#  --set keys.hydraSigning.existingSecret.key=hydra.sk \
#  --set keys.hydraVerification.existingConfigMap.name=hydra-verification \
#  --set keys.hydraVerification.items[0].filename=hydra.vk
# # helm upgrade hydra ./hydra-node \
# #  --namespace hydra \
# #  --set keys.hydraSigning.existingSecret.name=hydra-signing \
# #  --set keys.hydraSigning.existingSecret.key=hydra.sk \
# #  --set keys.hydraVerification.existingConfigMap.name=hydra-verification \
# #  --set keys.hydraVerification.items[0].filename=hydra.vk

# # Cardano node
helm install cardano-preview ./cardano-node/ \
  --namespace cardano-node \
  --set extraLabels.supernode/status=ready \
  --create-namespace \
  --set node.network=preview \
  --set node.networkMagic=2

# Midnight
kubectl create namespace midnight
openssl rand -hex 32 > /tmp/midnight-node.privatekey
kubectl -n midnight create secret generic midnight-node-key \
 --from-file=node.key=/tmp/midnight-node.privatekey
helm install midnight ./midnight/ \
  --namespace midnight \
  --values midnight-kind-values.yaml \
  --set extraLabels.supernode/status=ready \
  --set nodeKey.existingSecret.name=midnight-node-key \
  --set nodeKey.existingSecret.key=node.key

# helm upgrade midnight ./midnight/ \
#   --namespace midnight \
#   --values midnight-kind-values.yaml \
#   --set nodeKey.existingSecret.name=midnight-node-key \
#   --set nodeKey.existingSecret.key=node.key

# # Apex fusion
helm install prime-testnet ./apex-fusion \
  --namespace apex-fusion \
  --create-namespace \
  --set extraLabels.supernode/status=ready \
  --set node.network=prime-testnet
