import sys
import erpc
from erpc_shim import rpc_ble_api

class HostHandler(object):
    def __init__(self):
        pass

    def rpc_ble_init(self):
        print("rpc_ble_init")
    
    def rpc_ble_start(self):
        print("rpc_ble_start")
    
    def rpc_ble_deinit(self):
        print("rpc_ble_deinit")

class GapHandler(object):
    def __init__(self):
        pass
    def rpc_gap_set_param(self, param, value):
        print("rpc_gap_set_param: {}, {}".format(param, value))
        return param - 1

def main():
    transport = erpc.transport.TCPTransport("localhost", 5555, True)
    host_service = rpc_ble_api.server.rpc_ble_hostService(HostHandler())
    gap_service = rpc_ble_api.server.rpc_gapService(GapHandler())
    server = erpc.simple_server.SimpleServer(transport, erpc.basic_codec.BasicCodec)
    server.add_service(host_service)
    server.add_service(gap_service)
    print("Starting BLE Host Service.")
    sys.stdout.flush()

    server.run()

main()
