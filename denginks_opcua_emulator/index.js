process.env.NODEOPCUA_LOG_LEVEL = "Error";

const {
    OPCUAServer,
    Variant,
    DataType,
    MessageSecurityMode,
    SecurityPolicy,
    UserTokenType,
    StatusCodes
} = require("node-opcua");

async function createOPCUAServer() {
    const server = new OPCUAServer({
        port: 4840,
        resourcePath: "/UA/MyEmulator",
        buildInfo: {
            productName: "DENGINKS OPC-UA Emulator",
            buildNumber: "1.1.1",
            buildDate: new Date()
        },
        alternateHostNames: ["localhost", "127.0.0.1"],
        securityModes: [
            MessageSecurityMode.None,
            MessageSecurityMode.Sign,
            MessageSecurityMode.SignAndEncrypt
        ],
        securityPolicies: [
            SecurityPolicy.None,
            SecurityPolicy.Basic128Rsa15,
            SecurityPolicy.Basic256,
            SecurityPolicy.Basic256Sha256,
            SecurityPolicy.Aes128_Sha256_RsaOaep,
            SecurityPolicy.Aes256_Sha256_RsaPss
        ],
        allowAnonymous: true,
        userManager: {
            isValidUser: (userName, password) => {
                console.log(`[AUTH] User attempt: ${userName}`);
                if (userName === "admin" && password === "Denginks2026") {
                    console.log("[AUTH] Approved");
                    return true;
                }
                console.log(`[AUTH] Denied: ${userName}`);
                return false;
            }
        },
        onUserBind: (userToken) => {
            console.log(`[AUTH] Token: ${userToken.policyId}`);

            if (userToken.policyId === "anonymous") {
                console.log("[AUTH] OK");
                return StatusCodes.Good;
            }

            if (userToken.policyId === "username_password") {
                return StatusCodes.Good;
            }

            if (userToken.policyId === "certificate") {
                console.log("[AUTH] X509 OK");
                return StatusCodes.Good;
            }

            if (userToken.policyId === "issued_token") {
                const token = userToken.tokenData ? userToken.tokenData.toString() : "empty";
                console.log(`[AUTH] Token Data: ${token}`);
                return StatusCodes.Good;
            }

            console.log(`[AUTH] Locked: ${userToken.policyId}`);
            return StatusCodes.BadUserAccessDenied;
        }
    });

    server.on("new_session", (session) => {
        const id = session.nodeId ? session.nodeId.toString() : "N/A";
        console.log(`[SESSION] New: ${id}`);
    });

    server.on("session_activated", (session) => {
        const id = session.nodeId ? session.nodeId.toString() : "N/A";
        console.log(`[SESSION] Active: ${id}`);
    });

    server.on("session_closed", (session, reason) => {
        const id = session.nodeId ? session.nodeId.toString() : "N/A";
        console.log(`[SESSION] Closed: ${id} (Reason: ${reason})`);
    });

    server.on("connection_lost", () => {
        console.log("[CONNECT] Lost");
    });

    server.on("connection_reestablished", () => {
        console.log("[CONNECT] Restored");
    });

    await server.initialize();

    const addressSpace = server.engine.addressSpace;
    const namespace = addressSpace.getOwnNamespace();

    const production = namespace.addFolder(addressSpace.rootFolder.objects, {
        browseName: "Production"
    });

    const lines = ["Line1", "Line2"];
    lines.forEach(lineName => {
        const line = namespace.addFolder(production, { browseName: lineName });

        const modules = ["ModuleA", "ModuleB"];
        modules.forEach(moduleName => {
            const module = namespace.addFolder(line, { browseName: moduleName });

            const machines = ["Machine01", "Machine02"];
            machines.forEach(machineName => {
                const machine = namespace.addFolder(module, { browseName: machineName });
                const sensors = namespace.addFolder(machine, { browseName: "Sensors" });

                let temperatureValue = 20.0;
                namespace.addVariable({
                    componentOf: sensors,
                    browseName: "Temperature",
                    dataType: DataType.Double,
                    value: {
                        get: () => {
                            temperatureValue += (Math.random() - 0.5) * 0.5;
                            return new Variant({ dataType: DataType.Double, value: temperatureValue });
                        }
                    }
                });

                namespace.addVariable({
                    componentOf: sensors,
                    browseName: "Status",
                    dataType: DataType.Int32,
                    value: {
                        get: () => {
                            const val = Math.floor(Math.random() * 4);
                            return new Variant({ dataType: DataType.Int32, value: val });
                        }
                    }
                });

                let running = true;
                namespace.addVariable({
                    componentOf: sensors,
                    browseName: "IsRunning",
                    dataType: DataType.Boolean,
                    value: {
                        get: () => {
                            if (Math.random() > 0.95) running = !running;
                            return new Variant({ dataType: DataType.Boolean, value: running });
                        }
                    }
                });

                const messages = ["OK", "Warning", "Maint", "Stop"];
                namespace.addVariable({
                    componentOf: sensors,
                    browseName: "Message",
                    dataType: DataType.String,
                    value: {
                        get: () => {
                            const msg = messages[Math.floor(Math.random() * messages.length)];
                            return new Variant({ dataType: DataType.String, value: msg });
                        }
                    }
                });

                namespace.addVariable({
                    componentOf: sensors,
                    browseName: "LastUpdate",
                    dataType: DataType.DateTime,
                    value: {
                        get: () => new Variant({ dataType: DataType.DateTime, value: new Date() })
                    }
                });
            });
        });
    });

    await server.start();
    const endpointUrl = server.endpoints[0].endpointDescriptions()[0].endpointUrl;

    console.clear();
    console.log("=======================================================");
    console.log("       DENGINKS OPC-UA EMULATOR - PRO VERSION         ");
    console.log("=======================================================");
    console.log(` Endpoint URL : ${endpointUrl}`);
    console.log(" Status       : RUNNING");
    console.log("-------------------------------------------------------");
    console.log(" Authentication:");
    console.log(" - Anon, User (admin/Denginks2026), X509, IssuedTok");
    console.log("-------------------------------------------------------");
    console.log(" Security:");
    console.log(" - All Standard Policies Active");
    console.log("-------------------------------------------------------");
    console.log(" Hierarchy    : 5 levels");
    console.log("=======================================================");
}

createOPCUAServer().catch(err => {
    console.error("Critical Error:", err);
    process.exit(1);
});
