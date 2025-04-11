
let (let*) = Option.bind

type expr
  = Lit of char
  | Seq of expr list
  | Cho of expr list
  (* | Not of expr *)
  | Kle of expr
  | Emp
  | Rul of string

type grammar = (string * expr) list

type tree
  = Bud
  | Leaf of char
  | Node of tree list
  | Name of string * tree

let get_input (s : string) : (char * string) option =
  if String.length s = 0
  then None
  else Some (String.get s 0, String.sub s 1 (String.length s - 1))

let (++) (t : tree) (t' : tree) : tree =
  match t' with
  | Bud -> t
  | Node ts -> Node (t::ts)
  | t' -> Node [t; t']

let parse (g : grammar) (i : string) (s : string) : (tree *  string) option =
  let rec p (rn : string) (e : expr) (s : string) : (tree * string) option =
    match e with
    | Lit c ->
      let* hd, tl = get_input s in
      if c = hd
      then Some (Leaf c, tl)
      else None
    | Seq (r::rs) ->
      let* (t, s') = p rn r s in
      let* (t', s'') = p rn (Seq rs) s' in
      Some (t ++ t', s'')
    | Seq [] -> Some (Node [], s)
    | Cho (r::rs) ->
      Option.fold ~none: (p rn (Cho rs) s) ~some: (fun x -> Some x) (p rn r s)
    | Cho [] -> None
    | Kle e ->
      Option.fold
        ~none: (Some (Bud, s))
        ~some: (fun (t, s') ->
            let (t', s'') = Option.value (p rn (Kle e) s') ~default:(Bud, s') in
            (Some (t ++ t', s'')
            ))
        (p rn e s)
    (* | Not r' -> assert false *)
    | Emp -> Some (Bud, s)
    | Rul i' ->
      let* r = List.assoc_opt i' g in
      let* (t, s') = p i' r s in
      Some (Name (i', t), s')
  in
  p i (List.assoc i g) s


let prod_parse (g : grammar) (i : string) (s : string) : (tree *  string) option =
  let used : string list ref = ref [] in
  let rec p (rn : string) (e : expr) (s : string) : (tree * string) option =
    match e with
    | Lit c ->
      let* hd, tl = get_input s in
      if c = hd
      then (used := [] ; Some (Leaf c, tl))
      else None
    | Seq (r::rs) ->
      let* (t, s') = p rn r s in
      let* (t', s'') = p rn (Seq rs) s' in
      Some (t ++ t', s'')
    | Seq [] -> Some (Node [], s)
    | Cho (r::rs) ->
      Option.fold ~none: (p rn (Cho rs) s) ~some: (fun x -> Some x) (p rn r s)
    | Cho [] -> None
    | Kle e ->
      Option.fold
        ~none: (Some (Bud, s))
        ~some: (fun (t, s') ->
            if s = s'           (* it must consume input to be productive *)
            then Some (t, s')
            else let (t', s'') = Option.value (p rn (Kle e) s') ~default:(Bud, s') in
              (Some (t ++ t', s'')
            ))
        (p rn e s)
    (* | Not r' -> assert false *)
    | Emp -> Some (Bud, s)
    | Rul i' when not (List.mem i' !used) ->
      used := i::!used ;
      let* r = List.assoc_opt i' g in
      let* (t, s') = p i' r s in
      Some (Name (i', t), s')
    | Rul _ -> None (* the was used already and it could be unproductive *)
  in
  p i (Rul i) s

let expr : grammar = [("E", Cho [Seq[Rul "E"; Lit '+'; Rul "E"]; Seq[Rul "E"; Lit '*'; Rul "E"]; Lit '1'])]

let t0 = prod_parse expr "E" "1"
let t1 = prod_parse expr "E" "1+1"
let t2 = prod_parse expr "E" "1+1+1"
let t3 = prod_parse expr "E" "1+1*1"
let t4 = prod_parse expr "E" "1*1+1"

let g : grammar = [("R", Kle(Cho [Lit '1'; Lit '2'; Emp]))]

let t5 = prod_parse g "R" "1"
let t5b = prod_parse g "R" "2"
let t5bb = prod_parse g "R" "3"
let t6 = prod_parse g "R" "121"
let t7 = prod_parse g "R" ""
