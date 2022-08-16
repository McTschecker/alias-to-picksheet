<script setup lang="ts">
import { ref } from 'vue'
import { message, save } from "@tauri-apps/api/dialog"
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api'
defineProps<{ msg: string }>()

const count = ref(0)

let ready = ref(false)
let file = ""

const selectFile = async () => {
  while (true) {
    const input = await open({
      multiple: false,
      filters: [{
        name: "PDF",
        extensions: ["pdf"]
      }]
    });
    console.log(file);
    console.log(typeof file);

    if(input == null){
      await message('Bitte geben Sie eine Datei ein', { title: 'Falsche Eingabe', type: 'error' });
      break;
    }else{
      file = input;
      console.log(file);
      break;
    }
  }

}

let output_file = ""
const saveOutputPath = async () => {
  output_file = await save({
    title: "Save location of Picklist and Pickup Sheet",
    filters: [{
      name: 'PDF',
      extensions: ['pdf']
    }]
  });
  console.log(output_file)
}


const startGen = async () => {
  console.log(file + "  " + output_file);
  //await message("Opened File " + opened);
  if(file == null || output_file==null){
    await message('Bitte geben Sie eine Datei ein', { title: 'Falsche Eingabe', type: 'error' });
    return;
  }
  let res = await invoke("startPdf", { path: file, folder: false, outDir: output_file }).then((response) => console.log(response))
  //await message("Fertig", "PDF wurde fertiggestellt");

}
</script>

<template>

  <div class="card">
    {{ file }}
    <button type="button" @click="selectFile">Choose Input File</button>
  </div>

  <div class="card">
    {{ output_file }}
    <button type="button" @click="saveOutputPath">Choose Output</button>
  </div>

  <div class="card">
    <button type="button" :disabled="ready" @click="startGen">Start Generation</button>
  </div>

</template>

<style scoped>
.read-the-docs {
  color: #888;
}
</style>
