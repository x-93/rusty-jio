const jio = require('../../../../nodejs/jio');

jio.initConsolePanicHook();

(async () => {

    let encrypted = jio.encryptXChaCha20Poly1305("my message", "my_password");
    console.log("encrypted:", encrypted);
    let decrypted = jio.decryptXChaCha20Poly1305(encrypted, "my_password");
    console.log("decrypted:", decrypted);

})();
