for file in output/*"Coder Radio"*.json; do 
    filename=$(basename "$file")
    number="${filename#* | Coder Radio }"
    number="${number%.*}"
    number=$(printf "%05d" "$number")
    #cp "$file" /path/to/new/folder/"$number.json"; 
    echo "$number"
done
