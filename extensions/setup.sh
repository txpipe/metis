# Create K8s cluster
kind create cluster --name supernode --config config.yml
kubectl config use-context kind-supernode 

# Install control plane
helm install control-plane oci://supernode.store/control-plane \
  --version 0.2.0-rc1 \
  --namespace control-plane \
  --create-namespace

# # Dolos
# helm install dolos ./dolos \
#   --namespace dolos \
#   --create-namespace \
#   --values midnight-kind-values.yaml \
#   --set image.tag=v0.32.0 \
#   --set config.upstreamAddress=relay.cnode-m1.demeter.run:3002

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
# helm install cardano-node ./cardano-node/ \
#   --namespace cardano-node \
#   --create-namespace

# Midnight
kubectl create namespace midnight
openssl rand -hex 32 > /tmp/midnight-node.privatekey
kubectl -n midnight create secret generic midnight-node-key \
 --from-file=node.key=/tmp/midnight-node.privatekey
# helm install midnight oci://supernode.store/extensions/midnight \
#   --version 0.2.0-rc3 \
#   --namespace midnight \
#   --values midnight-kind-values.yaml \
#   --set nodeKey.existingSecret.name=midnight-node-key \
#   --set nodeKey.existingSecret.key=node.key
helm install midnight ./midnight/ \
  --namespace midnight \
  --values midnight-kind-values.yaml \
  --set nodeKey.existingSecret.name=midnight-node-key \
  --set nodeKey.existingSecret.key=node.key

# # Apex fusion
# helm install vector-tesnet ./apex-fusion \
#   --namespace apex-fusion \
#   --create-namespace \
#   --set node.network=vector-testnet \
#   --set node.networkMagic=1
#
# helm install prime-testnet ./apex-fusion \
#   --namespace apex-fusion \
#   --create-namespace \
#   --set node.network=prime-testnet \
#   --set node.networkMagic=1
#
# helm install prime-mainnet ./apex-fusion \
#   --namespace apex-fusion \
#   --create-namespace \
#   --set node.network=prime-mainnet \
#   --set node.networkMagic=1
