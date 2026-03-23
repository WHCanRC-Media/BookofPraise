melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d4 g4 a4 fis4 d4 g4 a4 b4 \break

  % Line 2
  a4 b4 g4 b4 cis4 d2 \break

  % Line 3
  a4 b4 a4 g4 fis4 e4 a4 fis4 \break

  % Line 4
  d4 d'4 c4 b4 a4 g2 \bar "|."
}
