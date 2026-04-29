melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 a'4 fis4 g4 a4 b4 b4 a4 \break

  % Line 2
  a4 d4 b4 cis4 cis4 b2 r4 \break

  % Line 3
  d4 cis4 b4 a4 a4 g4 fis4 e4 \break

  % Line 4
  a4 a4 d,4 fis4 e4 d1 \bar "|."
}
